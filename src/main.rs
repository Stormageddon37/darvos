use evdev::{Device, EventType};

fn main() -> std::io::Result<()> {
    let path = "/dev/input/event17";
    let mut device = Device::open(path)?;

    println!("Listening for key events from {path}...");
    let mut mic_enabled = false;

    loop {
        for ev in device.fetch_events()? {
            if ev.event_type() == EventType::ABSOLUTE {
                mic_enabled = ev.value() != 0;
            }
        }
        println!("Mic listening: {}", mic_enabled);
    }
}
