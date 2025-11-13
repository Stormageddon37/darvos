use crate::openrgb_client::kill_server;
use openrgb2::OpenRgbClient;
use std::process::exit;

pub fn validate_is_root() {
    if !nix::unistd::Uid::effective().is_root() {
        println!("Darvos must run as root");
        exit(1);
    }
}

pub async fn validate_server_running() -> OpenRgbClient {
    kill_server().await;
    crate::openrgb_client::start_server();
    let connected_client = crate::openrgb_client::connect_to_server().await;
    connected_client
}
