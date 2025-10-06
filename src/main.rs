use evdev::{Device, EventType};
use openrgb2::{Color, Controller, OpenRgbClient};
use std::net::TcpListener;
use std::process::{exit, Command, Stdio};
use std::result::Result;
use std::{env, io, path::PathBuf};
use tokio::time::Duration;

const RETRY_DELAY: Duration = Duration::from_secs(2);

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

async fn set_color(keyboard: &Controller, color: Color) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = keyboard.cmd();
    cmd.set_leds(vec![color; keyboard.num_leds()])?;
    cmd.execute().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !nix::unistd::Uid::effective().is_root() {
        println!("Darvos must run as root");
        exit(1);
    }


    let query = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: darvos <device-name-or-substring>");
        exit(2);
    });


    let path = loop {
        match find_device_path_by_name(&query) {
            Ok(path) => break path,
            Err(e) => {
                eprintln!(
                    "Failed to find device: {}. Retrying in {:?}...",
                    e, RETRY_DELAY
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    };
    println!("Using device {:?} from query {:?}", path, query);

    while is_port_in_use(6742) {
        Command::new("pkill")
            .arg("openrgb")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        tokio::time::sleep(RETRY_DELAY).await;
    }

    println!("Starting OpenRGB server...");
    Command::new("openrgb")
        .arg("--server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    println!("Connecting to OpenRGB server...");
    let client = loop {
        match OpenRgbClient::connect().await {
            Ok(client) => {
                println!("Connected!");
                break client
            },
            Err(e) => {
                eprintln!(
                    "{}. Retrying in {:?}...",
                    e, RETRY_DELAY
                );
                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    };
    let controllers = client.get_all_controllers().await?;
    controllers.init().await?;

    let mut device = Device::open(path)?;
    let mut mic_enabled = false; // Assume mic is on

    if let Some(keyboard) = controllers.into_iter().next() {
        let initial_color = Color::new(0, 0, 255);
        set_color(&keyboard, initial_color).await?;

        loop {
            for event in device.fetch_events()? {
                if event.event_type() == EventType::ABSOLUTE {
                    mic_enabled = event.value() != 0;
                }
            }
            let color = if mic_enabled {
                Color::new(0, 255, 0)
            } else {
                Color::new(255, 0, 0)
            };
            set_color(&keyboard, color).await?;
            println!("Set color: {}", color);
        }
    } else {
        println!("No keyboard found");
    }

    Ok(())
}
