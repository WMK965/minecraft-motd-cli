/// Bedrock Edition MOTD 协议实现
/// 通过 UDP RakNet Unconnected Ping 获取基岩版服务器信息

use std::fmt;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// MotdBE 信息
#[allow(dead_code)]
pub struct MotdBEInfo {
    /// 服务器状态 (online/offline)
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
    /// 存档名字
    pub level_name: String,
    /// 游戏模式
    pub game_mode: String,
    /// 服务器唯一 ID
    pub server_unique_id: String,
    /// 连接延迟 (毫秒)
    pub delay: i64,
}

impl MotdBEInfo {
    /// 创建一个离线状态的 MotdBEInfo
    fn offline() -> Self {
        Self {
            status: "offline".to_string(),
            host: String::new(),
            motd: String::new(),
            agreement: 0,
            version: String::new(),
            online: 0,
            max: 0,
            level_name: String::new(),
            game_mode: String::new(),
            server_unique_id: String::new(),
            delay: 0,
        }
    }
}

/// 详细的错误类型，便于 verbose 模式输出
#[derive(Debug)]
pub enum BedrockError {
    EmptyHost,
    SocketBind(std::io::Error),
    Connect(std::io::Error),
    SetTimeout(std::io::Error),
    SendFailed(std::io::Error),
    RecvFailed(std::io::Error),
    ResponseTooShort { received: usize },
    InsufficientFields { found: usize, expected: usize },
    ParseField { field: String, value: String, error: String },
}

impl fmt::Display for BedrockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BedrockError::EmptyHost => write!(f, "服务器地址为空"),
            BedrockError::SocketBind(e) => write!(f, "UDP Socket 绑定失败: {}", e),
            BedrockError::Connect(e) => write!(f, "UDP 连接失败 (端口可能不通): {}", e),
            BedrockError::SetTimeout(e) => write!(f, "设置超时失败: {}", e),
            BedrockError::SendFailed(e) => write!(f, "发送 Unconnected Ping 数据包失败: {}", e),
            BedrockError::RecvFailed(e) => write!(f, "接收响应超时或失败 (服务器未返回信息): {}", e),
            BedrockError::ResponseTooShort { received } =>
                write!(f, "服务器返回数据过短 (收到 {} 字节, 至少需要 34 字节)", received),
            BedrockError::InsufficientFields { found, expected } =>
                write!(f, "服务器返回字段不足 (收到 {} 个字段, 需要至少 {} 个)", found, expected),
            BedrockError::ParseField { field, value, error } =>
                write!(f, "解析字段 '{}' 失败: 值='{}', 错误={}", field, value, error),
        }
    }
}

impl std::error::Error for BedrockError {}

/// 通过 UDP 请求获取 MotdBE 信息
///
/// # Arguments
/// * `host` - 服务器地址，如 "nyan.xyz:19132"
/// * `verbose` - 是否输出详细调试信息
///
/// # Returns
/// MotdBEInfo 结构体
pub fn motd_be(host: &str, verbose: bool) -> Result<MotdBEInfo, BedrockError> {
    if host.is_empty() {
        return Err(BedrockError::EmptyHost);
    }

    if verbose {
        eprintln!("[verbose] BE: 正在创建 UDP socket...");
    }

    // 创建 UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(BedrockError::SocketBind)?;

    if verbose {
        eprintln!("[verbose] BE: 正在连接到 {}...", host);
    }

    socket.connect(host).map_err(BedrockError::Connect)?;
    socket.set_read_timeout(Some(Duration::from_secs(5))).map_err(BedrockError::SetTimeout)?;
    socket.set_write_timeout(Some(Duration::from_secs(5))).map_err(BedrockError::SetTimeout)?;

    // 组成发送数据
    let mut send_data: Vec<u8> = Vec::new();

    // Packet ID
    send_data.push(0x01);

    // 客户端发送时间 (8 bytes, big-endian)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    send_data.extend_from_slice(&now.to_be_bytes());

    // Magic Number (RakNet)
    send_data.extend_from_slice(&[
        0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD,
    ]);

    // 客户端 ID
    send_data.extend_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x00]);

    // 客户端 GUID (8 bytes, 全零)
    send_data.extend_from_slice(&0u64.to_be_bytes());

    if verbose {
        eprintln!("[verbose] BE: 发送 Unconnected Ping 包 ({} 字节)...", send_data.len());
    }

    // 发送数据并记录时间
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    socket.send(&send_data).map_err(BedrockError::SendFailed)?;

    if verbose {
        eprintln!("[verbose] BE: 等待 Unconnected Pong 响应 (超时: 5秒)...");
    }

    // 接收数据
    let mut buf = [0u8; 1024];
    let n = socket.recv(&mut buf).map_err(BedrockError::RecvFailed)?;

    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    if verbose {
        eprintln!("[verbose] BE: 收到响应 ({} 字节), 延迟 {}ms", n, end_time - start_time);
    }

    if n < 34 {
        return Err(BedrockError::ResponseTooShort { received: n });
    }

    // 解析数据：偏移 33 之后为服务器信息
    let server_info = &buf[33..n];
    // 去除末尾的 null 字节
    let server_info_str = String::from_utf8_lossy(server_info)
        .trim_end_matches('\0')
        .to_string();

    if verbose {
        eprintln!("[verbose] BE: 服务器信息原始数据: {}", server_info_str);
    }

    // 按 ; 分割数据
    let motd_data: Vec<&str> = server_info_str.split(';').collect();

    if motd_data.len() < 10 {
        return Err(BedrockError::InsufficientFields { found: motd_data.len(), expected: 10 });
    }

    // 解析各字段
    let motd1 = motd_data[1].to_string();
    let protocol_version: i32 = motd_data[2].parse().map_err(|e: std::num::ParseIntError| {
        BedrockError::ParseField {
            field: "protocol_version".to_string(),
            value: motd_data[2].to_string(),
            error: e.to_string(),
        }
    })?;
    let version_name = motd_data[3].to_string();
    let player_count: i32 = motd_data[4].parse().map_err(|e: std::num::ParseIntError| {
        BedrockError::ParseField {
            field: "player_count".to_string(),
            value: motd_data[4].to_string(),
            error: e.to_string(),
        }
    })?;
    let max_player_count: i32 = motd_data[5].parse().map_err(|e: std::num::ParseIntError| {
        BedrockError::ParseField {
            field: "max_player_count".to_string(),
            value: motd_data[5].to_string(),
            error: e.to_string(),
        }
    })?;
    let server_unique_id = motd_data[6].to_string();
    let motd2 = motd_data[7].to_string();
    let game_mode = motd_data[8].to_string();

    if verbose {
        eprintln!("[verbose] BE: 解析成功");
    }

    Ok(MotdBEInfo {
        status: "online".to_string(),
        host: host.to_string(),
        motd: motd1,
        agreement: protocol_version,
        version: version_name,
        online: player_count,
        max: max_player_count,
        level_name: motd2,
        game_mode,
        server_unique_id,
        delay: end_time - start_time,
    })
}
