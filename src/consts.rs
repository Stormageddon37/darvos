use openrgb2::Color;
use std::time::Duration;

pub const RETRY_DELAY: Duration = Duration::from_secs(2);
pub const RED: Color = Color::new(255, 0, 0);
pub const GREEN: Color = Color::new(0, 255, 0);
pub const BLUE: Color = Color::new(0, 0, 255);
pub const DEFAULT_COLOR: Color = BLUE;
