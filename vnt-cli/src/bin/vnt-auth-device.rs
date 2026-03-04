use console::style;
use std::sync::{Arc, Mutex};
use vnt::core::{Config, Vnt};
use vnt::{ErrorInfo, VntCallback};

#[derive(Clone, Default)]
struct AuthCallback {
    result: Arc<Mutex<Option<bool>>>,
}

impl VntCallback for AuthCallback {
    fn success(&self) {
        *self.result.lock().unwrap() = Some(true);
    }

    fn error(&self, info: ErrorInfo) {
        *self.result.lock().unwrap() = Some(false);
        println!("{}", style(format!("error {}", info)).red());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (server_addr, user_id, group, ticket) = match parse_args(&args) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            eprintln!("usage: vnt-auth-device [-s <server>] <user-id> <group> <ticket>");
            std::process::exit(2);
        }
    };
    if let Err(e) = run_auth_device(server_addr, user_id, group, ticket) {
        println!("{}", style(format!("Error {:?}", e)).red());
        std::process::exit(1);
    }
}

fn parse_args(args: &[String]) -> Result<(String, String, String, String), &'static str> {
    let default_server = "quic://controlmiddlescale.net:433".to_string();
    if args.len() == 4 {
        return Ok((
            default_server,
            args[1].clone(),
            args[2].clone(),
            args[3].clone(),
        ));
    }
    if args.len() == 6 && args[1] == "-s" {
        return Ok((
            args[2].clone(),
            args[3].clone(),
            args[4].clone(),
            args[5].clone(),
        ));
    }
    Err("invalid arguments")
}

fn run_auth_device(
    server_addr: String,
    user_id: String,
    group: String,
    ticket: String,
) -> anyhow::Result<()> {
    let mut config = Config::simple_new_config(
        common::config::get_device_id(),
        group.clone(),
        server_addr,
        None,
        None,
        None,
    )?;
    config.auth_user_id = Some(user_id);
    config.auth_group = Some(group);
    config.auth_ticket = Some(ticket);
    config.auth_only = true;
    let callback = AuthCallback::default();
    let vnt_util = Vnt::new(config, callback.clone())?;
    vnt_util.wait();
    match *callback.result.lock().unwrap() {
        Some(true) => println!("{}", style("auth result: success").green()),
        Some(false) => println!("{}", style("auth result: failed").red()),
        None => println!("{}", style("auth result: unknown").yellow()),
    }
    Ok(())
}
