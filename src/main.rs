use evdev::{Device, EventType};
use openrgb2::{Color, Controller, OpenRgbClient};
use std::process::{Command, Stdio};
use std::result::Result;
use std::{env, io, path::PathBuf};
use tokio::time::Duration;

use std::net::TcpListener;

fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}
fn find_device_path_by_name(device_query: &str) -> io::Result<PathBuf> {
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

async fn set_color(
    keyboard: &Controller,
    mic_enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let color = if mic_enabled {
        Color::new(0, 255, 0)
    } else {
        Color::new(255, 0, 0)
    };

    let mut cmd = keyboard.cmd();
    cmd.set_leds(vec![color; keyboard.num_leds()])?;
    cmd.execute().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: darvos <device-name-or-substring>");
        std::process::exit(2);
    });

    let max_retries = 6;
    let retry_delay = Duration::from_secs(10);
    
    let path = loop {
        match find_device_path_by_name(&query) {
            Ok(path) => break path,
            Err(e) if max_retries > 0 => {
                eprintln!("Failed to find device: {}. Retrying in {:?}...", e, retry_delay);
                tokio::time::sleep(retry_delay).await;
            }
            Err(e) => return Err(e.into()),
        }
    };
    println!("Using device: {:?}", path);

    if is_port_in_use(6742) {
        println!("OpenRGB server is already running");
    } else {
        Command::new("openrgb")
            .arg("--server")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        tokio::time::sleep(Duration::from_millis(10000)).await;
    }
    let client = OpenRgbClient::connect().await.unwrap_or_else(|_|
        panic!("Could not connect to OpenRGB server"));
    let controllers = client.get_all_controllers().await?;
    controllers.init().await?;

    let mut device = Device::open(path)?;
    let mut mic_enabled = false; // Assume mic is on

    if let Some(keyboard) = controllers.into_iter().next() {
        set_color(&keyboard, mic_enabled).await?;
        loop {
            for event in device.fetch_events()? {
                if event.event_type() == EventType::ABSOLUTE {
                    mic_enabled = event.value() != 0;
                }
            }
            set_color(&keyboard, mic_enabled).await?;
            println!("Set color: {}", mic_enabled);
        }
    } else {
        println!("No keyboard found");
    }

    Ok(())
}
