/// Java Edition MOTD 协议实现
/// 通过 TCP Server List Ping 获取 Java 版服务器信息
/// 参考: https://wiki.vg/Server_List_Ping

use serde::Deserialize;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// MotdJava 信息
#[allow(dead_code)]
pub struct MotdJavaInfo {
    /// 服务器状态
    pub status: String,
    /// 服务器地址
    pub host: String,
    /// Motd 信息
    pub motd: String,
    /// 协议版本
    pub agreement: i32,
    /// 支持的游戏版本
    pub version: String,
    /// 在线人数
    pub online: i32,
    /// 最大在线人数
    pub max: i32,
    /// 连接延迟 (毫秒)
    pub delay: i64,
}

impl MotdJavaInfo {
    /// 创建一个离线状态的 MotdJavaInfo
    fn offline() -> Self {
        Self {
            status: "offline".to_string(),
            host: String::new(),
            motd: String::new(),
            agreement: 0,
            version: String::new(),
            online: 0,
            max: 0,
            delay: 0,
        }
    }
}

/// 详细的错误类型，便于 verbose 模式输出
#[derive(Debug)]
pub enum JavaError {
    EmptyHost,
    DnsResolution(String),
    TcpConnect(std::io::Error),
    SetTimeout(std::io::Error),
    SendHandshake(std::io::Error),
    SendRequest(std::io::Error),
    ReadPacketLength(std::io::Error),
    ReadPacketId(std::io::Error),
    InvalidPacketId { expected: i32, received: i32 },
    ReadJsonLength(std::io::Error),
    ReadPayload(std::io::Error),
    JsonParse { raw: String, error: String },
    VarIntTooLarge,
}

impl fmt::Display for JavaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JavaError::EmptyHost => write!(f, "服务器地址为空"),
            JavaError::DnsResolution(e) => write!(f, "DNS 解析失败: {}", e),
            JavaError::TcpConnect(e) => write!(f, "TCP 连接失败 (端口可能不通): {}", e),
            JavaError::SetTimeout(e) => write!(f, "设置超时失败: {}", e),
            JavaError::SendHandshake(e) => write!(f, "发送握手包失败: {}", e),
            JavaError::SendRequest(e) => write!(f, "发送请求包失败: {}", e),
            JavaError::ReadPacketLength(e) => write!(f, "读取响应包长度失败 (服务器未返回信息): {}", e),
            JavaError::ReadPacketId(e) => write!(f, "读取包 ID 失败: {}", e),
            JavaError::InvalidPacketId { expected, received } =>
                write!(f, "收到无效的包 ID (期望 {}, 收到 {})", expected, received),
            JavaError::ReadJsonLength(e) => write!(f, "读取 JSON 长度失败: {}", e),
            JavaError::ReadPayload(e) => write!(f, "读取 JSON 数据失败: {}", e),
            JavaError::JsonParse { raw, error } =>
                write!(f, "JSON 解析失败: {}\n  原始数据: {}", error, &raw[..raw.len().min(200)]),
            JavaError::VarIntTooLarge => write!(f, "VarInt 超出范围"),
        }
    }
}

impl std::error::Error for JavaError {}

/// 原始 JSON 响应结构
#[derive(Deserialize)]
struct MotdJavaJson {
    description: serde_json::Value,
    players: PlayersInfo,
    version: VersionInfo,
}

#[derive(Deserialize)]
struct PlayersInfo {
    max: i32,
    online: i32,
}

#[derive(Deserialize)]
struct VersionInfo {
    name: String,
    protocol: i32,
}

/// 写入 VarInt 到 Vec 中
fn write_varint(buf: &mut Vec<u8>, value: i32) {
    let mut value = value as u32;
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// 从流中读取 VarInt
fn read_varint(stream: &mut TcpStream) -> Result<i32, JavaError> {
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).map_err(JavaError::ReadPacketLength)?;
        result |= ((byte[0] & 0x7F) as i32) << shift;
        if byte[0] & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(JavaError::VarIntTooLarge);
        }
    }
    Ok(result)
}

/// 通过 TCP 请求获取 Java 服务器信息
///
/// # Arguments
/// * `host` - 服务器地址，如 "nyan.xyz:25565"
/// * `verbose` - 是否输出详细调试信息
///
/// # Returns
/// MotdJavaInfo 结构体
pub fn motd_java(host: &str, verbose: bool) -> Result<MotdJavaInfo, JavaError> {
    if host.is_empty() {
        return Err(JavaError::EmptyHost);
    }

    let timeout = Duration::from_secs(5);

    if verbose {
        eprintln!("[verbose] JE: 正在解析地址 {}...", host);
    }

    // 分割地址
    let (address, port) = match host.rsplit_once(':') {
        Some((addr, port_str)) => (addr, port_str.parse::<u16>().unwrap_or(25565)),
        None => (host, 25565u16),
    };

    // DNS 解析
    use std::net::ToSocketAddrs;
    let socket_addr = (address, port)
        .to_socket_addrs()
        .map_err(|e| JavaError::DnsResolution(e.to_string()))?
        .next()
        .ok_or_else(|| JavaError::DnsResolution("no addresses found".to_string()))?;

    if verbose {
        eprintln!("[verbose] JE: DNS 解析结果: {}", socket_addr);
    }

    // 记录发送时间
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if verbose {
        eprintln!("[verbose] JE: 正在建立 TCP 连接 (超时: 5秒)...");
    }

    // 建立 TCP 连接
    let mut stream = TcpStream::connect_timeout(&socket_addr, timeout)
        .map_err(JavaError::TcpConnect)?;
    stream.set_read_timeout(Some(timeout)).map_err(JavaError::SetTimeout)?;
    stream.set_write_timeout(Some(timeout)).map_err(JavaError::SetTimeout)?;

    if verbose {
        eprintln!("[verbose] JE: TCP 连接已建立");
    }

    // 构造握手包
    let mut handshake_data: Vec<u8> = Vec::new();
    // Packet ID = 0x00
    write_varint(&mut handshake_data, 0x00);
    // Protocol Version (575 = 1.15.1，与原 Go 实现一致)
    write_varint(&mut handshake_data, 575);
    // Server Address (长度前缀 + 字符串)
    write_varint(&mut handshake_data, address.len() as i32);
    handshake_data.extend_from_slice(address.as_bytes());
    // Server Port (big-endian)
    handshake_data.extend_from_slice(&port.to_be_bytes());
    // Next State = 1 (Status)
    write_varint(&mut handshake_data, 1);

    // 包装为带长度前缀的包
    let mut handshake_packet: Vec<u8> = Vec::new();
    write_varint(&mut handshake_packet, handshake_data.len() as i32);
    handshake_packet.extend_from_slice(&handshake_data);

    if verbose {
        eprintln!("[verbose] JE: 发送握手包 ({} 字节)...", handshake_packet.len());
    }

    stream.write_all(&handshake_packet).map_err(JavaError::SendHandshake)?;

    // 发送请求包 (长度=1, PacketID=0x00)
    let request_packet: [u8; 2] = [0x01, 0x00];

    if verbose {
        eprintln!("[verbose] JE: 发送状态请求包...");
    }

    stream.write_all(&request_packet).map_err(JavaError::SendRequest)?;

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if verbose {
        eprintln!("[verbose] JE: 等待响应...");
    }

    // 读取响应包长度
    let packet_length = read_varint(&mut stream).map_err(|_| {
        JavaError::ReadPacketLength(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "failed to read packet length",
        ))
    })?;

    if verbose {
        eprintln!("[verbose] JE: 响应包长度: {} 字节", packet_length);
    }

    // 读取包 ID
    let packet_id = read_varint(&mut stream).map_err(|_| {
        JavaError::ReadPacketId(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "failed to read packet id",
        ))
    })?;
    if packet_id != 0 {
        return Err(JavaError::InvalidPacketId { expected: 0, received: packet_id });
    }

    // 读取 JSON 字符串长度
    let json_length = read_varint(&mut stream).map_err(|_| {
        JavaError::ReadJsonLength(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "failed to read json length",
        ))
    })? as usize;

    if verbose {
        eprintln!("[verbose] JE: JSON 数据长度: {} 字节", json_length);
    }

    // 读取 JSON 数据
    let mut payload = vec![0u8; json_length];
    stream.read_exact(&mut payload).map_err(JavaError::ReadPayload)?;

    let payload_str = String::from_utf8_lossy(&payload).to_string();

    if verbose {
        eprintln!("[verbose] JE: 原始 JSON: {}", &payload_str[..payload_str.len().min(500)]);
    }

    // 解析 JSON
    let resp: MotdJavaJson = serde_json::from_slice(&payload).map_err(|e| {
        JavaError::JsonParse {
            raw: payload_str.clone(),
            error: e.to_string(),
        }
    })?;

    // 解析 MOTD 文本
    // 支持三种格式：纯字符串, {"text": "..."}, {"translate": "..."}
    let motd_text = parse_description(&resp.description);

    if verbose {
        eprintln!("[verbose] JE: 解析成功, 延迟 {}ms", end_time - start_time);
    }

    Ok(MotdJavaInfo {
        status: "online".to_string(),
        host: host.to_string(),
        motd: motd_text,
        agreement: resp.version.protocol,
        version: resp.version.name,
        online: resp.players.online,
        max: resp.players.max,
        delay: end_time - start_time,
    })
}

/// 递归解析 Minecraft description 字段
/// 支持: 纯字符串, {"text": "..."}, {"translate": "..."}, {"extra": [...]}
fn parse_description(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => {
            let mut result = String::new();

            // 提取 text 或 translate
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                result.push_str(text);
            } else if let Some(translate) = obj.get("translate").and_then(|v| v.as_str()) {
                result.push_str(translate);
            }

            // 递归处理 extra 数组
            if let Some(extra) = obj.get("extra").and_then(|v| v.as_array()) {
                for item in extra {
                    // 处理每个 extra 元素的颜色/格式代码
                    let color_prefix = extract_color_prefix(item);
                    result.push_str(&color_prefix);
                    result.push_str(&parse_description(item));
                }
            }

            result
        }
        _ => String::new(),
    }
}

/// 从 JSON extra 元素中提取 Minecraft §颜色代码前缀
fn extract_color_prefix(value: &serde_json::Value) -> String {
    let mut prefix = String::new();
    if let Some(obj) = value.as_object() {
        // 颜色映射
        if let Some(color) = obj.get("color").and_then(|v| v.as_str()) {
            let code = match color {
                "black" => "§0",
                "dark_blue" => "§1",
                "dark_green" => "§2",
                "dark_aqua" => "§3",
                "dark_red" => "§4",
                "dark_purple" => "§5",
                "gold" => "§6",
                "gray" => "§7",
                "dark_gray" => "§8",
                "blue" => "§9",
                "green" => "§a",
                "aqua" => "§b",
                "red" => "§c",
                "light_purple" => "§d",
                "yellow" => "§e",
                "white" => "§f",
                _ => "",
            };
            prefix.push_str(code);
        }
        // 格式代码
        if obj.get("bold").and_then(|v| v.as_bool()).unwrap_or(false) {
            prefix.push_str("§l");
        }
        if obj.get("italic").and_then(|v| v.as_bool()).unwrap_or(false) {
            prefix.push_str("§o");
        }
        if obj.get("underlined").and_then(|v| v.as_bool()).unwrap_or(false) {
            prefix.push_str("§n");
        }
        if obj.get("strikethrough").and_then(|v| v.as_bool()).unwrap_or(false) {
            prefix.push_str("§m");
        }
        if obj.get("obfuscated").and_then(|v| v.as_bool()).unwrap_or(false) {
            prefix.push_str("§k");
        }
    }
    prefix
}
