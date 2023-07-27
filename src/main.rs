use crate::error::Result;
use tokio::select;
use upower::Monitor;

mod error;
mod upower;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let monitor = Monitor::new().await?;

  select! {
    _ = handle_changes(&monitor) => {},
    _ = monitor.run() => {},
  }

  Ok(())
}

async fn handle_changes(monitor: &Monitor) -> Result<()> {
  loop {
    let state = monitor.changed_state().await?;
    dbg!(state);
  }
}
