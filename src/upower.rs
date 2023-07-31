use std::time::Duration;

use tokio::{
  sync::{
    watch::{self, Receiver, Sender},
    RwLock,
  },
  time,
};
use tokio_stream::StreamExt as TokioStream;
use upower_dbus::{DeviceProxy, UPowerProxy};
use zbus::export::futures_util::stream::{self, StreamExt};

use crate::{
  state::{PowerStatus, State},
  Result,
};

pub struct Monitor {
  device: DeviceProxy<'static>,
  receiver: RwLock<Receiver<State>>,
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
    let receiver = RwLock::new(receiver);

    Ok(Self {
      device,
      receiver,
      sender,
    })
  }

  pub async fn run(&self) -> Result<()> {
    macro_rules! event {
      ($name:ident) => {
        StreamExt::map(self.device.$name().await, |_| stringify!($name)).boxed()
      };
    }

    let events = [
      event!(receive_energy_changed),
      event!(receive_percentage_changed),
      event!(receive_state_changed),
      event!(receive_battery_level_changed),
    ];

    // force update at least almost every 30 seconds
    let min_poll_interval = time::interval(Duration::from_secs(30));
    let mut updates = TokioStream::timeout_repeating(
      stream::select_all(events),
      min_poll_interval,
    );

    while StreamExt::next(&mut updates).await.is_some() {
      let state = get_state(&self.device).await?;
      self.sender.send(state).expect("receiver dropped")
    }

    Ok(())
  }

  pub async fn changed_state(&self) -> Result<State> {
    let mut recv_mut = self.receiver.write().await;
    let _ = recv_mut.changed().await;
    drop(recv_mut);
    Ok(self.receiver.read().await.borrow().clone())
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

  let state = match device.state().await? {
    Charging => PowerStatus::Charging { time_to_full },
    Discharging => PowerStatus::Discharging { time_to_empty },
    FullyCharged => PowerStatus::FullyCharged,
    PendingCharge => PowerStatus::NotCharging,
    _ => PowerStatus::Unknown,
  };

  Ok(State {
    percentage,
    wattage,
    status: state,
  })
}
