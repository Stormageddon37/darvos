mod consts;
mod utils;
mod validators;

use crate::consts::DEFAULT_COLOR;
use crate::utils::{retry_find_device_path_by_name, select_color};
use crate::validators::{validate_is_root, validate_server_running};
use evdev::{Device, EventType};
use openrgb2::{Color, Controller};
use std::env;
use std::error::Error;
use std::process::exit;
use std::result::Result;

async fn get_microphone() -> Result<Device, Box<dyn Error>> {
    let query = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: darvos <device-name-or-substring>");
        exit(2);
    });

    let device_path = retry_find_device_path_by_name(&*query).await?;
    println!("Using device {:?} from query {:?}", device_path, query);
    let microphone = Device::open(device_path)?;
    Ok(microphone)
}

async fn get_keyboard() -> Result<Controller, Box<dyn Error>> {
    let rgb_client = validate_server_running().await;
    let controllers = rgb_client.get_all_controllers().await?;
    controllers.init().await?;

    let keyboard = controllers.into_iter().next().ok_or("No keyboard found")?;
    Ok(keyboard)
}

async fn set_keyboard_color(keyboard: &Controller, color: Color) -> Result<(), Box<dyn Error>> {
    let mut cmd = keyboard.cmd();
    cmd.set_leds(vec![color; keyboard.num_leds()])?;
    cmd.execute().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    validate_is_root();

    let mut microphone = get_microphone().await?;
    let mut mic_enabled = false; // Assume mic is off

    let keyboard = get_keyboard().await?;
    println!("Using keyboard {:?}", keyboard.name());
    set_keyboard_color(&keyboard, DEFAULT_COLOR).await?;

    loop {
        for event in microphone.fetch_events()? {
            if event.event_type() == EventType::ABSOLUTE {
                mic_enabled = event.value() != 0;
            }
        }
        println!(
            "{}",
            if mic_enabled {
                "Mic is unmuted"
            } else {
                "Mic is muted"
            }
        );
        let color = select_color(mic_enabled);
        set_keyboard_color(&keyboard, color).await?;
    }
}
