use crate::error::Result;
use tokio::select;
use upower::Monitor;

mod error;
mod state;
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
  let mut last_output = None;
  loop {
    let state = monitor.changed_state().await?;
    let new_output = serde_json::to_string(&state)?;

    match last_output {
      Some(ref last_output) if last_output == &new_output => {}
      _ => {
        println!("{}", new_output);
        last_output = Some(new_output);
      }
    }
  }
}
