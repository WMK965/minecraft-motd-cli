/// 地址工具模块
/// 提供服务器地址的端口补全、分割以及 Minecraft 彩色字符解析

/// 判断是否有端口，如果没有则自动补全
///
/// # Arguments
/// * `address` - 服务器地址，可带端口也可不带
/// * `default_port` - 需要补全的默认端口
///
/// # Returns
/// 补全后的地址字符串
pub fn complete_port(address: &str, default_port: u16) -> String {
    if address.contains(':') {
        address.to_string()
    } else {
        format!("{}:{}", address, default_port)
    }
}

/// 分割服务器地址与端口
///
/// # Arguments
/// * `address` - 服务器地址（带端口）
///
/// # Returns
/// (主机名, 端口号) 元组
pub fn split_address(address: &str) -> (&str, u16) {
    match address.rsplit_once(':') {
        Some((host, port_str)) => {
            let port = port_str.parse::<u16>().unwrap_or(0);
            (host, port)
        }
        None => (address, 0),
    }
}

/// 将 Minecraft § 颜色代码转换为 ANSI 终端转义序列
///
/// 支持的颜色代码:
/// §0-§f: 颜色, §l: 粗体, §m: 删除线, §n: 下划线, §o: 斜体, §r: 重置, §k: 混淆(跳过)
pub fn mc_color_to_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '§' || ch == '\u{00A7}' {
            if let Some(&code) = chars.peek() {
                chars.next(); // consume the code character
                let ansi = match code {
                    '0' => "\x1b[30m",   // Black
                    '1' => "\x1b[34m",   // Dark Blue
                    '2' => "\x1b[32m",   // Dark Green
                    '3' => "\x1b[36m",   // Dark Aqua
                    '4' => "\x1b[31m",   // Dark Red
                    '5' => "\x1b[35m",   // Dark Purple
                    '6' => "\x1b[33m",   // Gold
                    '7' => "\x1b[37m",   // Gray
                    '8' => "\x1b[90m",   // Dark Gray
                    '9' => "\x1b[94m",   // Blue
                    'a' | 'A' => "\x1b[92m",   // Green
                    'b' | 'B' => "\x1b[96m",   // Aqua
                    'c' | 'C' => "\x1b[91m",   // Red
                    'd' | 'D' => "\x1b[95m",   // Light Purple
                    'e' | 'E' => "\x1b[93m",   // Yellow
                    'f' | 'F' => "\x1b[97m",   // White
                    'l' | 'L' => "\x1b[1m",    // Bold
                    'm' | 'M' => "\x1b[9m",    // Strikethrough
                    'n' | 'N' => "\x1b[4m",    // Underline
                    'o' | 'O' => "\x1b[3m",    // Italic
                    'r' | 'R' => "\x1b[0m",    // Reset
                    'k' | 'K' => "",           // Obfuscated (skip)
                    _ => {
                        // Unknown code, output as-is
                        result.push('§');
                        result.push(code);
                        continue;
                    }
                };
                result.push_str(ansi);
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    // 确保末尾重置
    if result.contains("\x1b[") {
        result.push_str("\x1b[0m");
    }

    result
}

/// 去除 Minecraft § 颜色代码，返回纯文本
pub fn strip_mc_color(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '§' || ch == '\u{00A7}' {
            // 跳过下一个字符（颜色代码）
            chars.next();
        } else {
            result.push(ch);
        }
    }

    result
}

/// 检测终端是否支持 ANSI 颜色
pub fn supports_ansi_color() -> bool {
    // Windows 10+ 的现代终端通常支持 ANSI
    // 检查 NO_COLOR 环境变量 (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // 检查 TERM 环境变量
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Windows: 尝试启用虚拟终端处理
    #[cfg(windows)]
    {
        enable_windows_ansi();
    }

    true
}

/// Windows 下启用 ANSI 转义序列支持
#[cfg(windows)]
fn enable_windows_ansi() {
    use std::os::windows::io::AsRawHandle;

    unsafe {
        let handle = std::io::stdout().as_raw_handle();
        let mut mode: u32 = 0;

        // 外部函数声明
        extern "system" {
            fn GetConsoleMode(handle: *mut std::ffi::c_void, mode: *mut u32) -> i32;
            fn SetConsoleMode(handle: *mut std::ffi::c_void, mode: u32) -> i32;
        }

        if GetConsoleMode(handle as _, &mut mode) != 0 {
            // ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004
            let _ = SetConsoleMode(handle as _, mode | 0x0004);
        }
    }
}
