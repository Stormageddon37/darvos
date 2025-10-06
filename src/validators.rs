use crate::consts::RETRY_DELAY;
use crate::utils::is_port_in_use;
use openrgb2::OpenRgbClient;
use std::process::{Command, Stdio, exit};

pub fn validate_is_root() {
    if !nix::unistd::Uid::effective().is_root() {
        println!("Darvos must run as root");
        exit(1);
    }
}

pub async fn validate_server_running() -> OpenRgbClient {
    while is_port_in_use(6742) {
        Command::new("pkill")
            .arg("openrgb")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok();
        println!("Waiting for OpenRGB server to exit...");
        tokio::time::sleep(RETRY_DELAY).await;
    }

    println!("Starting OpenRGB server...");
    Command::new("openrgb")
        .arg("--server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok();
    println!("Started!");

    println!("Connecting to OpenRGB server...");
    let client = loop {
        match OpenRgbClient::connect().await {
            Ok(client) => {
                println!("Connected!");
                break client;
            }
            Err(e) => {
                eprintln!("{}. Retrying in {:?}...", e, RETRY_DELAY);
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    };
    client
}
