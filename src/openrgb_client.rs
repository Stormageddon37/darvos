use crate::consts::{OPENRGB_PORT, RETRY_DELAY};
use crate::utils::is_port_in_use;
use openrgb2::OpenRgbClient;
use std::process::{Command, Stdio};

pub async fn kill_server() {
    println!("Killing openrgb process...");
    while is_port_in_use(OPENRGB_PORT) {
        Command::new("pkill")
            .arg("openrgb")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok();
        println!("Waiting for OpenRGB server to die...");
        tokio::time::sleep(RETRY_DELAY).await;
    }
    println!("OpenRGB server killed successfully.");
}

pub fn start_server() {
    println!("Starting OpenRGB server...");
    Command::new("openrgb")
        .arg("--server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok();
    println!("Started!");
}

pub async fn connect_to_server() -> OpenRgbClient {
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
