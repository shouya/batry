use std::{
  cell::RefCell,
  time::{Duration, Instant},
};

use tokio::sync::watch::{self, Receiver, Sender};
use upower_dbus::{DeviceProxy, UPowerProxy};
use zbus::export::futures_util::{stream, StreamExt};

use crate::Result;

#[derive(Debug, Clone)]
pub struct State {
  pub percentage: f64,
  pub wattage: f64,
  pub state: BatteryState,
  pub updated_at: Instant,
}

#[derive(Debug, Clone)]
pub enum BatteryState {
  Discharging { time_to_empty: Duration },
  Charging { time_to_full: Duration },
  FullyCharged,
  // AC connected but not charging into battery because it's already
  // above some threshold, i.e. the battery is not expect to go empty.
  NotCharging,
  Unknown,
}

pub struct Monitor {
  device: DeviceProxy<'static>,
  receiver: RefCell<Receiver<State>>,
  sender: Sender<State>,
}

/*
Sample readout:

{'BatteryLevel': 1,
 'Capacity': 100.0,
 'ChargeCycles': 28,
 'Energy': 76.47,
 'EnergyEmpty': 0.0,
 'EnergyFull': 95.04,
 'EnergyFullDesign': 90.09,
 'EnergyRate': 38.763,
 'HasHistory': True,
 'HasStatistics': True,
 'IconName': 'battery-full-charging-symbolic',
 'IsPresent': True,
 'IsRechargeable': True,
 'Luminosity': 0.0,
 'Model': '5B11B79217',
 'NativePath': 'BAT0',
 'Online': False,
 'Percentage': 80.0,
 'PowerSupply': True,
 'Serial': '236',
 'State': 5,
 'Technology': 2,
 'Temperature': 0.0,
 'TimeToEmpty': 0,
 'TimeToFull': 0,
 'Type': 2,
 'UpdateTime': 1690380294,
 'Vendor': 'SMP',
 'Voltage': 16.821,
 'WarningLevel': 1}
 */

impl Monitor {
  pub async fn new() -> Result<Monitor> {
    let connection = zbus::Connection::system().await?;
    let device = UPowerProxy::new(&connection)
      .await?
      .get_display_device()
      .await?;

    let current_state = get_state(&device).await?;
    let (sender, receiver) = watch::channel(current_state);
    let receiver = RefCell::new(receiver);

    Ok(Self {
      device,
      receiver,
      sender,
    })
  }

  pub async fn run(&self) -> Result<()> {
    macro_rules! event {
      ($name:ident) => {
        self.device.$name().await.map(|_| stringify!($name)).boxed()
      };
    }

    let events = [
      event!(receive_energy_changed),
      event!(receive_percentage_changed),
      event!(receive_state_changed),
      event!(receive_battery_level_changed),
    ];

    let mut updates = stream::select_all(events);
    while let Some(event_name) = updates.next().await {
      dbg!(event_name);
      let state = get_state(&self.device).await?;
      self.sender.send(state).expect("receiver dropped")
    }

    Ok(())
  }

  pub async fn changed_state(&self) -> Result<State> {
    let _ = self.receiver.borrow_mut().changed().await;
    Ok(self.receiver.borrow().borrow().clone())
  }
}

async fn get_state(device: &DeviceProxy<'_>) -> Result<State> {
  use upower_dbus::BatteryState::{
    Charging, Discharging, FullyCharged, PendingCharge,
  };

  let percentage = device.percentage().await?;
  let wattage = device.get_property("EnergyRate").await?;
  let time_to_full = device
    .get_property::<i64>("TimeToFull")
    .await
    .map(|x| Duration::from_secs(x as u64))?;

  let time_to_empty = device
    .get_property::<i64>("TimeToEmpty")
    .await
    .map(|x| Duration::from_secs(x as u64))?;
  let timestamp = Instant::now();

  let state = match device.state().await? {
    Charging => BatteryState::Charging { time_to_full },
    Discharging => BatteryState::Discharging { time_to_empty },
    FullyCharged => BatteryState::FullyCharged,
    PendingCharge => BatteryState::NotCharging,
    _ => BatteryState::Unknown,
  };

  Ok(State {
    percentage,
    wattage,
    state,
    updated_at: timestamp,
  })
}
