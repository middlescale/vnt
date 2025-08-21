use clap::Parser;
use serde::Deserialize;
use std::fs;

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
    /// IP 地址
    #[arg(long)]
    ip: Option<String>,
    /// 端口号
    #[arg(long)]
    port: Option<u16>,
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

    let ip = args.ip.or_else(|| file_config.as_ref()?.ip.clone());
    let port = args.port.or_else(|| file_config.as_ref()?.port);

    println!("Server: {}", server);
    println!("Token: {}", token);
    if let Some(ip) = ip {
        println!("IP: {}", ip);
    }
    if let Some(port) = port {
        println!("Port: {}", port);
    }
}
