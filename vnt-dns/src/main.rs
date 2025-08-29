use clap::Parser;
use common::config;
use serde::Deserialize;
use std::fs;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use trust_dns_server::ServerFuture;
use trust_dns_server::authority::Catalog;
use trust_dns_server::proto::rr::Name;
use trust_dns_server::proto::rr::RData;
use trust_dns_server::proto::rr::Record;
use trust_dns_server::proto::rr::rdata::A;
use trust_dns_server::store::in_memory::InMemoryAuthority;
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
    /// 可选，IP 地址
    #[arg(long)]
    ip: Option<String>,
    /// 必选，端口号
    #[arg(long)]
    port: u16,
    /// 配置文件路径，默认为当前目录下的 config.yaml
    #[arg(long, default_value = "./config.yaml")]
    config: String,
    /// 可选，虚拟网卡名称
    #[arg(long)]
    nic: Option<String>,
}

/// 配置文件结构体
#[derive(Deserialize, Debug)]
struct FileConfig {
    server: Option<String>,
    token: Option<String>,
    ip: Option<String>,
    port: Option<u16>,
    nic: Option<String>,
}

async fn run_dns_server(ip: Ipv4Addr) {
    // 创建一个内存 Authority
    let origin = Name::from_ascii("aliyun-hk.ms.net.").unwrap();
    let authority = InMemoryAuthority::empty(
        origin.clone(),
        trust_dns_server::authority::ZoneType::Primary,
        false,
    );

    // 添加一条 A 记录
    let record = Record::from_rdata(
        origin.clone(),
        3600,
        RData::A(A::new(10, 26, 0, 6)), // 用 RData::A 包裹
    );
    if !authority.upsert(record, 0).await {
        println!("DNS authority doesnot upserted");
    }

    // 创建 Catalog 并注册 Authority
    let mut catalog = Catalog::new();
    catalog.upsert(origin.clone().into(), Box::new(Arc::new(authority)));

    // 启动 DNS 服务器
    let mut server = ServerFuture::new(catalog);
    let listen_addr = SocketAddr::new(std::net::IpAddr::V4(ip), 53);
    let udp_socket = tokio::net::UdpSocket::bind(listen_addr).await.unwrap();
    server.register_socket(udp_socket);

    println!("DNS server listening on {}", listen_addr);
    server.block_until_done().await.unwrap();
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

    // ip 可选，优先命令行，否则配置文件，否则 None
    let ip = if let Some(ip_arg) = &args.ip {
        ip_arg.clone()
    } else if let Some(cfg) = file_config.as_ref() {
        if let Some(ip_str) = &cfg.ip {
            ip_str.clone()
        } else {
            String::new()
        }
    } else {
        String::new()
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

    // 新增：nic 参数处理
    let nic = if let Some(nic_arg) = &args.nic {
        Some(nic_arg.clone())
    } else if let Some(cfg) = file_config.as_ref() {
        cfg.nic.clone()
    } else {
        None
    };

    println!("Server: {}", server);
    println!("Token: {}", token);
    if !ip.is_empty() {
        println!("IP: {}", ip);
    }
    println!("Port: {}", port);
    if let Some(nic) = &nic {
        println!("NIC: {}", nic);
    }

    let mut ip = if !ip.is_empty() {
        match ip.parse::<Ipv4Addr>() {
            Ok(addr) => Some(addr),
            Err(e) => {
                println!("IP 解析失败: {:?}", e);
                None
            }
        }
    } else {
        None
    };

    let device_id = config::get_device_id();
    if device_id.is_empty() {
        println!("获取 device_id 失败");
        return;
    }
    println!("Device ID: {}", device_id);

    // 传递 nic 参数到 Config::simple_new_config（如 API 支持）
    let config =
        match Config::simple_new_config(device_id, token, server, ip, Some(vec![port]), nic) {
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

    if ip.is_none() {
        // 等待服务器分配 IP
        use std::time::{Duration, Instant};
        let start = Instant::now();
        loop {
            let info = vnt_util.current_device_info().load();
            if matches!(info.status, vnt::handle::ConnectStatus::Connected)
                && info.virtual_ip() != Ipv4Addr::new(0, 0, 0, 0)
            {
                ip = Some(info.virtual_ip());
                println!("分配到 IP: {}", info.virtual_ip());
                break;
            }
            if start.elapsed() > Duration::from_secs(10) {
                println!("等待服务器分配 IP 超时");
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    // DNS server退出通知
    let dns_notify = Arc::new(Notify::new());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let vnt_c = vnt_util.clone();
            let dns_notify_c = dns_notify.clone();
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
                            dns_notify_c.notify_one();
                            handle.close();
                            break;
                        }
                        _ => {}
                    }
                }
            });
        }

        // 启动 DNS server
        if let Some(ip_addr) = ip {
            tokio::spawn(run_dns_server(ip_addr));
        } else {
            println!("DNS server 未启动，IP 不可用");
        }

        vnt_util.wait();
    });
}
