/// motd-cli - 像 PING 一样获取我的世界服务器 MOTD 信息
///
/// 基于 MCBE-Server-Motd 项目使用 Rust 重写的命令行版本

mod protocol;
mod util;
mod i18n;

use std::env;
use std::net::ToSocketAddrs;

/// 打印帮助信息
fn print_help(i18n: &i18n::I18n) {
    println!("motd-cli - {}", i18n.help_desc);
    println!();
    println!("{}", i18n.help_usage);
    println!("{}", i18n.help_usage_1);
    println!("{}", i18n.help_usage_2);
    println!();
    println!("{}", i18n.help_args);
    println!("{}", i18n.help_arg_be);
    println!("{}", i18n.help_arg_je);
    println!();
    println!("{}", i18n.help_opts);
    println!("{}", i18n.help_opt_h);
    println!("{}", i18n.help_opt_v);
    println!("{}", i18n.help_opt_j);
    println!();
    println!("{}", i18n.help_examples);
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
fn get_be_serve(address: &str, verbose: bool, use_color: bool, json_out: bool, i18n: &i18n::I18n) {
    let address = util::complete_port(address, 19132);
    let (host, port) = util::split_address(&address);
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let ip = addr.ip();
                if !json_out {
                    if host == ip.to_string() {
                        println!("{}", i18n.fetching.replace("{}", &address));
                    } else {
                        println!("{}", i18n.fetching_ip.replacen("{}", &address, 1).replacen("{}", &ip.to_string(), 1).replacen("{}", &port.to_string(), 1));
                    }
                }
            }
        }
        Err(e) => {
            if json_out {
                println!(r#"{{"status": "offline", "error": "{}"}}"#, e);
            } else {
                println!("{}", i18n.connect_fail);
            }
            if verbose {
                eprintln!("[verbose] {}: {}", i18n.dns_fail, e);
            }
            return;
        }
    }

    if !json_out {
        println!();
    }

    match protocol::bedrock::motd_be(&address, verbose) {
        Ok(motd) => {
            if json_out {
                let mut json_motd = motd;
                if !use_color {
                    json_motd.motd = util::strip_mc_color(&json_motd.motd);
                    json_motd.level_name = util::strip_mc_color(&json_motd.level_name);
                }
                println!("{}", serde_json::to_string(&json_motd).unwrap());
            } else {
                if motd.status == "offline" {
                    println!("{}", i18n.offline);
                } else {
                    println!("  {}:      {}ms", i18n.delay, motd.delay);
                    println!("  {}:  {}/{}", i18n.online, motd.online, motd.max);
                    println!("  {}:  {}", i18n.protocol, motd.agreement);
                    println!("  {}:  {}", i18n.version, motd.version);
                    println!("  {}:  {}", i18n.level_name, format_mc_text(&motd.level_name, use_color));
                    println!("  {}:  {}", i18n.game_mode, motd.game_mode);
                    println!("  {}:      {}", i18n.motd, format_mc_text(&motd.motd, use_color));
                }
            }
        }
        Err(e) => {
            if json_out {
                println!(r#"{{"status": "offline", "error": "{}"}}"#, e);
            } else {
                println!("{}", i18n.connect_fail);
            }
            if verbose {
                eprintln!("[verbose] 错误详情: {}", e);
            }
        }
    }
}

/// 打印 Java 版服务器状态（多行格式）
fn get_je_serve(address: &str, verbose: bool, use_color: bool, json_out: bool, i18n: &i18n::I18n) {
    let address = util::complete_port(address, 25565);
    let (host, port) = util::split_address(&address);
    match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let ip = addr.ip();
                if !json_out {
                    if host == ip.to_string() {
                        println!("{}", i18n.fetching.replace("{}", &address));
                    } else {
                        println!("{}", i18n.fetching_ip.replacen("{}", &address, 1).replacen("{}", &ip.to_string(), 1).replacen("{}", &port.to_string(), 1));
                    }
                }
            }
        }
        Err(e) => {
            if json_out {
                println!(r#"{{"status": "offline", "error": "{}"}}"#, e);
            } else {
                println!("{}", i18n.connect_fail);
            }
            if verbose {
                eprintln!("[verbose] {}: {}", i18n.dns_fail, e);
            }
            return;
        }
    }

    if !json_out {
        println!();
    }

    match protocol::java::motd_java(&address, verbose) {
        Ok(motd) => {
            if json_out {
                let mut json_motd = motd;
                if !use_color {
                    json_motd.motd = util::strip_mc_color(&json_motd.motd);
                }
                println!("{}", serde_json::to_string(&json_motd).unwrap());
            } else {
                if motd.status == "offline" {
                    println!("{}", i18n.offline);
                } else {
                    println!("  {}:      {}ms", i18n.delay, motd.delay);
                    println!("  {}:  {}/{}", i18n.online, motd.online, motd.max);
                    println!("  {}:  {}", i18n.protocol, motd.agreement);
                    println!("  {}:  {}", i18n.version, motd.version);
                    println!("  {}:      {}", i18n.motd, format_mc_text(&motd.motd, use_color));
                }
            }
        }
        Err(e) => {
            if json_out {
                println!(r#"{{"status": "offline", "error": "{}"}}"#, e);
            } else {
                println!("{}", i18n.connect_fail);
            }
            if verbose {
                eprintln!("[verbose] 错误详情: {}", e);
            }
        }
    }
}

/// 推断模式：先尝试 BE，再尝试 JE
fn guess(address: &str, verbose: bool, use_color: bool, json_out: bool, i18n: &i18n::I18n) {
    let be_address = util::complete_port(address, 19132);
    let (host, port) = util::split_address(&be_address);

    let ip_str = match (host, port).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                addr.ip().to_string()
            } else {
                if json_out {
                    println!(r#"{{"status": "offline", "error": "DNS fallback"}}"#);
                } else {
                    println!("{}", i18n.connect_fail);
                }
                return;
            }
        }
        Err(e) => {
            if json_out {
                println!(r#"{{"status": "offline", "error": "{}"}}"#, e);
            } else {
                println!("{}", i18n.connect_fail);
            }
            if verbose {
                eprintln!("[verbose] {}: {}", i18n.dns_fail, e);
            }
            return;
        }
    };

    let print_trying_tips = |addr: &str, host: &str, ip: &str, port: u16| {
        if !json_out {
            if host == ip {
                println!("{}", i18n.fetching.replace("{}", addr));
            } else {
                println!("{}", i18n.fetching_ip.replacen("{}", addr, 1).replacen("{}", ip, 1).replacen("{}", &port.to_string(), 1));
            }
        }
    };

    if verbose {
        eprintln!("[verbose] {}", i18n.guess_be);
    }

    match protocol::bedrock::motd_be(&be_address, verbose) {
        Ok(motd) if motd.status != "offline" => {
            print_trying_tips(&be_address, host, &ip_str, port);
            if json_out {
                let mut json_motd = motd;
                if !use_color {
                    json_motd.motd = util::strip_mc_color(&json_motd.motd);
                    json_motd.level_name = util::strip_mc_color(&json_motd.level_name);
                }
                println!("{}", serde_json::to_string(&json_motd).unwrap());
            } else {
                println!();
                println!("  类型:      {}", i18n.type_be);
                println!("  {}:      {}ms", i18n.delay, motd.delay);
                println!("  {}:  {}/{}", i18n.online, motd.online, motd.max);
                println!("  {}:  {}", i18n.protocol, motd.agreement);
                println!("  {}:  {}", i18n.version, motd.version);
                println!("  {}:  {}", i18n.level_name, format_mc_text(&motd.level_name, use_color));
                println!("  {}:  {}", i18n.game_mode, motd.game_mode);
                println!("  {}:      {}", i18n.motd, format_mc_text(&motd.motd, use_color));
            }
            return;
        }
        Ok(_) => {
            if verbose {
                eprintln!("[verbose] {}", i18n.be_offline);
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("[verbose] {}: {}", i18n.be_fail, e);
                eprintln!("[verbose] {}", i18n.guess_je);
            }
        }
    }

    let je_address = util::complete_port(address, 25565);
    let (_, je_port) = util::split_address(&je_address);

    match protocol::java::motd_java(&je_address, verbose) {
        Ok(motd) if motd.status != "offline" => {
            print_trying_tips(&je_address, host, &ip_str, je_port);
            if json_out {
                let mut json_motd = motd;
                if !use_color {
                    json_motd.motd = util::strip_mc_color(&json_motd.motd);
                }
                println!("{}", serde_json::to_string(&json_motd).unwrap());
            } else {
                println!();
                println!("  类型:      {}", i18n.type_je);
                println!("  {}:      {}ms", i18n.delay, motd.delay);
                println!("  {}:  {}/{}", i18n.online, motd.online, motd.max);
                println!("  {}:  {}", i18n.protocol, motd.agreement);
                println!("  {}:  {}", i18n.version, motd.version);
                println!("  {}:      {}", i18n.motd, format_mc_text(&motd.motd, use_color));
            }
            return;
        }
        Ok(_) => {
            if verbose {
                eprintln!("[verbose] {}", i18n.je_offline);
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("[verbose] {}: {}", i18n.je_fail, e);
            }
        }
    }

    if json_out {
        println!(r#"{{"status": "offline", "error": "Both BE and JE offline"}}"#);
    } else {
        println!("{}", i18n.connect_fail);
    }
}

/// 解析命令行参数
struct Args {
    verbose: bool,
    json: bool,
    version: Option<String>,
    address: Option<String>,
    show_help: bool,
}

fn parse_args() -> Args {
    let raw_args: Vec<String> = env::args().skip(1).collect();

    let mut args = Args {
        verbose: false,
        json: false,
        version: None,
        address: None,
        show_help: false,
    };

    let mut positional: Vec<String> = Vec::new();

    for arg in &raw_args {
        match arg.as_str() {
            "-h" | "--help" => args.show_help = true,
            "-v" | "--verbose" => args.verbose = true,
            "-j" | "--json" => args.json = true,
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
    let i18n_lang = i18n::get_i18n();

    if args.show_help || args.address.is_none() {
        print_help(i18n_lang);
        return;
    }

    let address = args.address.unwrap();
    // JSON output doesn't need ANSI styling unless requested, but raw Minecraft styling might be wanted or unwanted.
    // The user will usually prefer no ANSI colors in terminal. We strip color format in json serialization if use_color is true? 
    // Usually scripts parse plain text if they use json. Let's make use_color=false force strip color.
    let use_color = if args.json { false } else { util::supports_ansi_color() };

    if args.verbose {
        eprintln!("[verbose] 终端颜色支持: {}", if args.json { false } else { util::supports_ansi_color() });
    }

    match args.version.as_deref() {
        Some("be") => get_be_serve(&address, args.verbose, use_color, args.json, i18n_lang),
        Some("je") => get_je_serve(&address, args.verbose, use_color, args.json, i18n_lang),
        _ => guess(&address, args.verbose, use_color, args.json, i18n_lang),
    }
}
