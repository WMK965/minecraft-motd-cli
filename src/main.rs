/// motd-cli - 像 PING 一样获取我的世界服务器 MOTD 信息
///
/// 基于 MCBE-Server-Motd 项目使用 Rust 重写的命令行版本

mod protocol;
mod util;

use std::env;
use std::net::ToSocketAddrs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 打印帮助信息
fn print_help() {
    println!("motd-cli v{} - 像 PING 一样获取我的世界服务器 MOTD 信息", VERSION);
    println!();
    println!("用法:");
    println!("  motd-cli <be|je> <地址[:端口]>    指定服务器类型查询");
    println!("  motd-cli <地址[:端口]>            自动推断服务器类型");
    println!();
    println!("参数:");
    println!("  be                 基岩版服务器 (默认端口: 19132)");
    println!("  je                 Java版服务器 (默认端口: 25565)");
    println!();
    println!("选项:");
    println!("  -h, --help         显示此帮助信息");
    println!("  -v, --verbose      显示详细调试信息");
    println!();
    println!("示例:");
    println!("  motd-cli be play.craftersmc.net:19132");
    println!("  motd-cli je play.hypixel.net");
    println!("  motd-cli play.craftersmc.net");
    println!("  motd-cli -v be play.craftersmc.net");
}

/// 格式化彩色文本字段，根据终端支持情况处理 § 颜色代码
fn format_mc_text(text: &str, use_color: bool) -> String {
    if use_color {
        util::mc_color_to_ansi(text)
    } else {
        util::strip_mc_color(text)
    }
}

/// 打印基岩版服务器状态（多行格式）
fn get_be_serve(address: &str, verbose: bool, use_color: bool) {
    // 补全端口
    let address = util::complete_port(address, 19132);

    // 获取 IP 地址
    let (host, port) = util::split_address(&address);
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let ip = addr.ip();
                if host == ip.to_string() {
                    println!("正在获取 {} 的 MOTD 信息...", address);
                } else {
                    println!("正在获取 {} [{}:{}] 的 MOTD 信息...", address, ip, port);
                }
            }
        }
        Err(e) => {
            println!("无法连接到服务器，请检查地址是否正确");
            if verbose {
                eprintln!("[verbose] DNS 解析失败: {}", e);
            }
            return;
        }
    }

    println!();

    // 获取服务器信息
    match protocol::bedrock::motd_be(&address, verbose) {
        Ok(motd) => {
            if motd.status == "offline" {
                println!("服务器离线");
            } else {
                println!("  延迟:      {}ms", motd.delay);
                println!("  在线人数:  {}/{}", motd.online, motd.max);
                println!("  协议版本:  {}", motd.agreement);
                println!("  游戏版本:  {}", motd.version);
                println!("  地图名称:  {}", format_mc_text(&motd.level_name, use_color));
                println!("  游戏模式:  {}", motd.game_mode);
                println!("  MOTD:      {}", format_mc_text(&motd.motd, use_color));
            }
        }
        Err(e) => {
            println!("无法连接到服务器，请检查地址是否正确");
            if verbose {
                eprintln!("[verbose] 错误详情: {}", e);
            }
        }
    }
}

/// 打印 Java 版服务器状态（多行格式）
fn get_je_serve(address: &str, verbose: bool, use_color: bool) {
    // 补全端口
    let address = util::complete_port(address, 25565);

    // 获取 IP 地址
    let (host, port) = util::split_address(&address);
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let ip = addr.ip();
                if host == ip.to_string() {
                    println!("正在获取 {} 的 MOTD 信息...", address);
                } else {
                    println!("正在获取 {} [{}:{}] 的 MOTD 信息...", address, ip, port);
                }
            }
        }
        Err(e) => {
            println!("无法连接到服务器，请检查地址是否正确");
            if verbose {
                eprintln!("[verbose] DNS 解析失败: {}", e);
            }
            return;
        }
    }

    println!();

    // 获取服务器信息
    match protocol::java::motd_java(&address, verbose) {
        Ok(motd) => {
            if motd.status == "offline" {
                println!("服务器离线");
            } else {
                println!("  延迟:      {}ms", motd.delay);
                println!("  在线人数:  {}/{}", motd.online, motd.max);
                println!("  协议版本:  {}", motd.agreement);
                println!("  游戏版本:  {}", motd.version);
                println!("  MOTD:      {}", format_mc_text(&motd.motd, use_color));
            }
        }
        Err(e) => {
            println!("无法连接到服务器，请检查地址是否正确");
            if verbose {
                eprintln!("[verbose] 错误详情: {}", e);
            }
        }
    }
}

/// 推断模式：先尝试 BE，再尝试 JE
fn guess(address: &str, verbose: bool, use_color: bool) {
    // 尝试基岩版
    let be_address = util::complete_port(address, 19132);
    let (host, port) = util::split_address(&be_address);

    let ip_str = match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                addr.ip().to_string()
            } else {
                println!("无法连接到服务器，请检查地址是否正确");
                return;
            }
        }
        Err(e) => {
            println!("无法连接到服务器，请检查地址是否正确");
            if verbose {
                eprintln!("[verbose] DNS 解析失败: {}", e);
            }
            return;
        }
    };

    let print_trying_tips = |addr: &str, host: &str, ip: &str, port: u16| {
        if host == ip {
            println!("正在获取 {} 的 MOTD 信息...", addr);
        } else {
            println!("正在获取 {} [{}:{}] 的 MOTD 信息...", addr, ip, port);
        }
    };

    if verbose {
        eprintln!("[verbose] 推断模式: 先尝试基岩版...");
    }

    // 获取 BE 服务器信息
    match protocol::bedrock::motd_be(&be_address, verbose) {
        Ok(be_motd) if be_motd.status != "offline" => {
            print_trying_tips(&be_address, host, &ip_str, port);
            println!();
            println!("  类型:      Bedrock Edition");
            println!("  延迟:      {}ms", be_motd.delay);
            println!("  在线人数:  {}/{}", be_motd.online, be_motd.max);
            println!("  协议版本:  {}", be_motd.agreement);
            println!("  游戏版本:  {}", be_motd.version);
            println!("  地图名称:  {}", format_mc_text(&be_motd.level_name, use_color));
            println!("  游戏模式:  {}", be_motd.game_mode);
            println!("  MOTD:      {}", format_mc_text(&be_motd.motd, use_color));
            return;
        }
        Ok(_) => {
            if verbose {
                eprintln!("[verbose] 基岩版返回离线状态，尝试 Java 版...");
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("[verbose] 基岩版查询失败: {}", e);
                eprintln!("[verbose] 尝试 Java 版...");
            }
        }
    }

    // 尝试 Java 版
    let je_address = util::complete_port(address, 25565);
    let (_, je_port) = util::split_address(&je_address);

    match protocol::java::motd_java(&je_address, verbose) {
        Ok(je_motd) if je_motd.status != "offline" => {
            print_trying_tips(&je_address, host, &ip_str, je_port);
            println!();
            println!("  类型:      Java Edition");
            println!("  延迟:      {}ms", je_motd.delay);
            println!("  在线人数:  {}/{}", je_motd.online, je_motd.max);
            println!("  协议版本:  {}", je_motd.agreement);
            println!("  游戏版本:  {}", je_motd.version);
            println!("  MOTD:      {}", format_mc_text(&je_motd.motd, use_color));
            return;
        }
        Ok(_) => {
            if verbose {
                eprintln!("[verbose] Java 版返回离线状态");
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("[verbose] Java 版查询也失败: {}", e);
            }
        }
    }

    println!("无法连接到服务器，请检查地址是否正确");
}

/// 解析命令行参数
struct Args {
    verbose: bool,
    version: Option<String>, // "be", "je", or None for guess
    address: Option<String>,
    show_help: bool,
}

fn parse_args() -> Args {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    let mut args = Args {
        verbose: false,
        version: None,
        address: None,
        show_help: false,
    };

    let mut positional: Vec<String> = Vec::new();

    for arg in &raw_args {
        match arg.as_str() {
            "-h" | "--help" => args.show_help = true,
            "-v" | "--verbose" => args.verbose = true,
            _ => positional.push(arg.clone()),
        }
    }

    if positional.is_empty() {
        return args;
    }

    if positional[0] == "be" || positional[0] == "je" {
        args.version = Some(positional[0].clone());
        if positional.len() > 1 {
            args.address = Some(positional[1].clone());
        }
    } else {
        args.address = Some(positional[0].clone());
    }

    args
}

fn main() {
    let args = parse_args();

    if args.show_help || args.address.is_none() {
        print_help();
        return;
    }

    let address = args.address.unwrap();
    let use_color = util::supports_ansi_color();

    if args.verbose {
        eprintln!("[verbose] 终端颜色支持: {}", use_color);
    }

    match args.version.as_deref() {
        Some("be") => get_be_serve(&address, args.verbose, use_color),
        Some("je") => get_je_serve(&address, args.verbose, use_color),
        _ => guess(&address, args.verbose, use_color),
    }
}
