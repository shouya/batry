use std::str::FromStr;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Deserialize, Debug)]
enum PowerSupplyStatus {
  Charging,
  Discharging,
  Full,
  Unknown,
}

#[derive(Deserialize, Debug)]
pub struct UEvent {
  name: String,
  status: PowerSupplyStatus,
  energy_full: u64,
  energy_now: u64,
}

impl UEvent {
  pub fn is_discharging(&self) -> bool {
    matches!(self.status, PowerSupplyStatus::Discharging)
  }

  pub fn percentage(&self) -> u64 {
    (self.energy_now * 100 / self.energy_full).clamp(0, 100)
  }
}

impl FromStr for UEvent {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let iter = s.lines().map(|line| {
      let (key, value) = line.split_once('=').unwrap();
      (key.to_string(), value.to_string())
    });

    let uevent = envy::prefixed("POWER_SUPPLY_").from_iter(iter)?;
    Ok(uevent)
  }
}
