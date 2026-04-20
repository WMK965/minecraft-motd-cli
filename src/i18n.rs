#[cfg(not(windows))]
use std::env;

pub struct I18n {
    pub help_desc: &'static str,
    pub help_usage: &'static str,
    pub help_usage_1: &'static str,
    pub help_usage_2: &'static str,
    pub help_args: &'static str,
    pub help_arg_be: &'static str,
    pub help_arg_je: &'static str,
    pub help_opts: &'static str,
    pub help_opt_h: &'static str,
    pub help_opt_v: &'static str,
    pub help_opt_j: &'static str,
    pub help_examples: &'static str,
    
    pub fetching: &'static str,
    pub fetching_ip: &'static str,
    pub connect_fail: &'static str,
    pub dns_fail: &'static str,
    
    pub offline: &'static str,
    pub delay: &'static str,
    pub online: &'static str,
    pub protocol: &'static str,
    pub version: &'static str,
    pub level_name: &'static str,
    pub game_mode: &'static str,
    pub motd: &'static str,
    
    pub guess_be: &'static str,
    pub be_offline: &'static str,
    pub be_fail: &'static str,
    pub guess_je: &'static str,
    pub je_offline: &'static str,
    pub je_fail: &'static str,
    
    pub type_be: &'static str,
    pub type_je: &'static str,
}

pub const ZH: I18n = I18n {
    help_desc: "像 PING 一样获取我的世界服务器 MOTD 信息",
    help_usage: "用法:",
    help_usage_1: "  motd-cli <be|je> <地址[:端口]>    指定服务器类型查询",
    help_usage_2: "  motd-cli <地址[:端口]>            自动推断服务器类型",
    help_args: "参数:",
    help_arg_be: "  be                 基岩版服务器 (默认端口: 19132)",
    help_arg_je: "  je                 Java版服务器 (默认端口: 25565)",
    help_opts: "选项:",
    help_opt_h: "  -h, --help         显示此帮助信息",
    help_opt_v: "  -v, --verbose      显示详细调试信息",
    help_opt_j: "  -j, --json         以 JSON 格式输出结果",
    help_examples: "示例:",
    
    fetching: "正在获取 {} 的 MOTD 信息...",
    fetching_ip: "正在获取 {} [{}:{}] 的 MOTD 信息...",
    connect_fail: "无法连接到服务器，请检查地址是否正确",
    dns_fail: "DNS 解析失败",
    
    offline: "服务器离线",
    delay: "延迟",
    online: "在线人数",
    protocol: "协议版本",
    version: "游戏版本",
    level_name: "地图名称",
    game_mode: "游戏模式",
    motd: "MOTD",
    
    guess_be: "推断模式: 先尝试基岩版...",
    be_offline: "基岩版返回离线状态，尝试 Java 版...",
    be_fail: "基岩版查询失败",
    guess_je: "尝试 Java 版...",
    je_offline: "Java 版返回离线状态",
    je_fail: "Java 版查询也失败",
    
    type_be: "Bedrock Edition",
    type_je: "Java Edition",
};

pub const EN: I18n = I18n {
    help_desc: "Get Minecraft server MOTD info like PING",
    help_usage: "Usage:",
    help_usage_1: "  motd-cli <be|je> <address[:port]>    Query specific server type",
    help_usage_2: "  motd-cli <address[:port]>            Auto-guess server type",
    help_args: "Arguments:",
    help_arg_be: "  be                 Bedrock Edition (default port: 19132)",
    help_arg_je: "  je                 Java Edition (default port: 25565)",
    help_opts: "Options:",
    help_opt_h: "  -h, --help         Show this help message",
    help_opt_v: "  -v, --verbose      Show verbose debug info",
    help_opt_j: "  -j, --json         Output in JSON format",
    help_examples: "Examples:",
    
    fetching: "Fetching MOTD info for {}...",
    fetching_ip: "Fetching MOTD info for {} [{}:{}]...",
    connect_fail: "Failed to connect to the server, please check the address",
    dns_fail: "DNS resolution failed",
    
    offline: "Server Offline",
    delay: "Delay",
    online: "Online",
    protocol: "Protocol",
    version: "Version",
    level_name: "Level Name",
    game_mode: "Game Mode",
    motd: "MOTD",
    
    guess_be: "Guess mode: Trying Bedrock Edition first...",
    be_offline: "Bedrock Edition is offline, trying Java Edition...",
    be_fail: "Bedrock Edition query failed",
    guess_je: "Trying Java Edition...",
    je_offline: "Java Edition is offline",
    je_fail: "Java Edition query failed",
    
    type_be: "Bedrock Edition",
    type_je: "Java Edition",
};

#[cfg(windows)]
fn is_chinese_supported() -> bool {
    extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
        fn GetConsoleOutputCP() -> u32;
    }
    unsafe {
        let lang = GetUserDefaultUILanguage();
        let cp = GetConsoleOutputCP();
        // zh-CN is 0x0804, zh-TW is 0x0404, etc. Primary lang ID for Chinese is 0x04.
        let is_zh = (lang & 0x03FF) == 0x04;
        // 65001 = UTF-8, 936 = GBK, 54936 = GB18030, 950 = Big5
        let is_supported_cp = cp == 65001 || cp == 936 || cp == 54936 || cp == 950;
        is_zh && is_supported_cp
    }
}

#[cfg(not(windows))]
fn is_chinese_supported() -> bool {
    let lang = env::var("LANG").unwrap_or_default().to_lowercase();
    let lc_all = env::var("LC_ALL").unwrap_or_default().to_lowercase();
    let is_zh = lang.contains("zh") || lc_all.contains("zh");
    
    let is_utf8 = lang.contains("utf-8") || lang.contains("utf8") || lc_all.contains("utf-8") || lc_all.contains("utf8") || lang == "c.utf-8" || lang.is_empty();

    let tz = env::var("TZ").unwrap_or_default();
    let is_zh_tz = tz.contains("Shanghai") || tz.contains("Chongqing") || tz.contains("Taipei") || tz.contains("Hong_Kong") || tz.contains("Macau");
    
    (is_zh || is_zh_tz) && is_utf8
}

pub fn get_i18n() -> &'static I18n {
    if is_chinese_supported() {
        &ZH
    } else {
        &EN
    }
}
