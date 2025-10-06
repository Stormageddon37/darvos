use crate::consts::{GREEN, RED, RETRY_DELAY};
use openrgb2::Color;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;

pub fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

pub async fn retry_find_device_path_by_name(device_query: &str) -> io::Result<PathBuf> {
    loop {
        match find_device_path_by_name(&device_query) {
            Ok(path) => break Ok(path),
            Err(e) => {
                eprintln!(
                    "Failed to find device: {}. Retrying in {:?}...",
                    e, RETRY_DELAY
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }
}

pub fn find_device_path_by_name(device_query: &str) -> io::Result<PathBuf> {
    let device_query_lower = device_query.to_lowercase();

    let mut best: Option<PathBuf> = None;

    for (path, dev) in evdev::enumerate() {
        if let Some(name) = dev.name() {
            let name_lower = name.to_lowercase();

            if name_lower == device_query_lower {
                return Ok(path);
            }

            if best.is_none() && name_lower.contains(&device_query_lower) {
                best = Some(path);
            }
        }
    }

    best.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("No /dev/input/event* device found with name matching '{device_query}'"),
        )
    })
}

pub fn select_color(mic_enabled: bool) -> Color {
    if mic_enabled { GREEN } else { RED }
}
