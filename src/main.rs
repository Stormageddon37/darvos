mod consts;
mod openrgb_client;
mod utils;
mod validators;

use crate::consts::DEFAULT_COLOR;
use crate::openrgb_client::kill_server;
use crate::utils::{retry_find_device_path_by_name, select_color};
use crate::validators::{validate_is_root, validate_server_running};
use evdev::{Device, EventType};
use futures::FutureExt;
use openrgb2::{Color, Controller};
use std::env;
use std::error::Error;
use std::panic::AssertUnwindSafe;
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

async fn cleanup() {
    println!("Cleaning up...");
    kill_server().await;
    println!("Clean up complete.");
}

async fn init() -> Result<(), Box<dyn Error>> {
    let mut microphone = get_microphone().await?;
    let mut mic_enabled = true; // Mic is on when plugged in

    let keyboard = get_keyboard().await?;
    println!("Using keyboard {:?}", keyboard.name());
    set_keyboard_color(&keyboard, DEFAULT_COLOR).await?;

    let mut search_mic = false;
    loop {
        if search_mic {
            microphone = get_microphone().await?;
            search_mic = false;
        }
        match microphone.fetch_events() {
            Ok(events) => {
                for event in events {
                    if event.event_type() == EventType::ABSOLUTE {
                        mic_enabled = event.value() != 0;
                    }
                }
                let color = select_color(mic_enabled);
                set_keyboard_color(&keyboard, color).await?;
            }
            Err(err) => {
                eprintln!("Failed to fetch events: {err}");
                search_mic = true;
            }
        };
    }
}

#[tokio::main]
async fn main() {
    validate_is_root();
    let _ = AssertUnwindSafe(init()).catch_unwind().await;
    cleanup().await
}
