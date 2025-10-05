use evdev::{Device, EventType};
use openrgb2::{Color, Controller, OpenRgbClient};
use std::result::Result;
use std::{env, io, path::PathBuf};
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

async fn set_color(keyboard: &Controller, mic_enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
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
    let needle = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: darvos <device-name-or-substring>");
        std::process::exit(2);
    });

    let path = find_device_path_by_name(&needle)?;
    println!("Using device: {:?}", path);

    let client = OpenRgbClient::connect().await?;
    let controllers = client.get_all_controllers().await?;
    controllers.init().await?;

    let mut device = Device::open(path)?;
    let mut mic_enabled = true; // Assume mic is on

    if let Some(keyboard) = controllers.into_iter().next() {
        set_color(&keyboard, mic_enabled).await?;
        loop {
            for event in device.fetch_events()? {
                if event.event_type() == EventType::ABSOLUTE {
                    mic_enabled = event.value() != 0;
                }
            }
            set_color(&keyboard, mic_enabled).await?;
        }
    }

    Ok(())
}
