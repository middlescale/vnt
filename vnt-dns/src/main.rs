use clap::Parser;
use common::config;
use serde::Deserialize;
use std::fs;
use std::net::Ipv4Addr;
use vnt::core::Config;

/// 命令行参数结构体
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// 必选，服务器地址
    #[arg(short = 's', long)]
    server: String,
    /// 必选，令牌
    #[arg(short = 'k', long)]
    token: String,
    /// 必选，IP 地址
    #[arg(long)]
    ip: String,
    /// 必选，端口号
    #[arg(long)]
    port: u16,
    /// 配置文件路径，默认为当前目录下的 config.yaml
    #[arg(long, default_value = "./config.yaml")]
    config: String,
}

/// 配置文件结构体
#[derive(Deserialize, Debug)]
struct FileConfig {
    server: Option<String>,
    token: Option<String>,
    ip: Option<String>,
    port: Option<u16>,
}

fn main() {
    let args = Args::parse();

    // 读取配置文件（始终尝试读取）
    let file_config = match fs::read_to_string(&args.config) {
        Ok(content) => match serde_json::from_str::<FileConfig>(&content) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                println!("配置文件解析失败: {:?}", e);
                None
            }
        },
        Err(e) => {
            println!("读取配置文件失败: {:?}", e);
            None
        }
    };

    // 优先使用命令行参数，否则用配置文件
    let server = if !args.server.is_empty() {
        args.server.clone()
    } else if let Some(cfg) = file_config.as_ref() {
        if let Some(s) = &cfg.server {
            s.clone()
        } else {
            println!("请通过命令行或配置文件指定 server");
            return;
        }
    } else {
        println!("请通过命令行或配置文件指定 server");
        return;
    };

    let token = if !args.token.is_empty() {
        args.token.clone()
    } else if let Some(cfg) = file_config.as_ref() {
        if let Some(t) = &cfg.token {
            t.clone()
        } else {
            println!("请通过命令行或配置文件指定 token");
            return;
        }
    } else {
        println!("请通过命令行或配置文件指定 token");
        return;
    };

    // ip 和 port 必选，优先命令行，否则配置文件，否则报错
    let ip = if !args.ip.is_empty() {
        args.ip.clone()
    } else if let Some(cfg) = file_config.as_ref() {
        if let Some(ip_str) = &cfg.ip {
            ip_str.clone()
        } else {
            println!("请通过命令行或配置文件指定 ip");
            return;
        }
    } else {
        println!("请通过命令行或配置文件指定 ip");
        return;
    };

    let port = if args.port != 0 {
        args.port
    } else if let Some(cfg) = file_config.as_ref() {
        if let Some(p) = cfg.port {
            p
        } else {
            println!("请通过命令行或配置文件指定 port");
            return;
        }
    } else {
        println!("请通过命令行或配置文件指定 port");
        return;
    };

    println!("Server: {}", server);
    println!("Token: {}", token);
    println!("IP: {}", ip);
    println!("Port: {}", port);

    let ip = match ip.parse::<Ipv4Addr>() {
        Ok(addr) => Some(addr),
        Err(e) => {
            println!("IP 解析失败: {:?}", e);
            None
        }
    };

    let device_id = config::get_device_id();
    if device_id.is_empty() {
        println!("获取 device_id 失败");
        return;
    }

    let config = match Config::simple_new_config(device_id, token, server, ip, Some(vec![port])) {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("创建配置失败: {:?}", e);
            return;
        }
    };

    // 创建 Vnt 实例并启动
    let vnt_util = match vnt::core::Vnt::new(config, common::callback::VntHandler {}) {
        Ok(vnt) => vnt,
        Err(e) => {
            println!("vnt create error: {:?}", e);
            std::process::exit(1);
        }
    };

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let vnt_c = vnt_util.clone();
        let mut signals = signal_hook::iterator::Signals::new([
            signal_hook::consts::SIGINT,
            signal_hook::consts::SIGTERM,
        ])
        .unwrap();
        let handle = signals.handle();
        std::thread::spawn(move || {
            for sig in signals.forever() {
                match sig {
                    signal_hook::consts::SIGINT | signal_hook::consts::SIGTERM => {
                        println!("Received SIGINT, {}", sig);
                        vnt_c.stop();
                        handle.close();
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    vnt_util.wait();
}
