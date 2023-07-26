use std::{
  fs::File,
  io::{Read, Seek, SeekFrom},
  time::Duration,
};

use crate::{error::Result, uevent::UEvent};

mod error;
mod uevent;

const BATTERY_PATH: &str = "/sys/class/power_supply/BAT0/uevent";

fn main() -> Result<()> {
  let mut file = File::open(BATTERY_PATH)?;
  let mut buffer = String::new();

  loop {
    file.seek(SeekFrom::Start(0))?;
    file.read_to_string(&mut buffer)?;

    let uevent: UEvent = buffer.parse()?;

    if uevent.percentage() < 10 && uevent.is_discharging() {
      send_low_battery_notification(uevent.percentage())?;
    }

    std::thread::sleep(Duration::from_secs(5));
  }

  Ok(())
}

fn send_low_battery_notification(percent: u64) -> Result<()> {
  std::process::Command::new("notify-send")
    .arg("Battery is low!")
    .arg(format!("{}%", percent))
    .arg("-u")
    .arg("critical")
    .spawn()?;

  Ok(())
}
