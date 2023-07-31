use clap::Parser;
use tokio::select;

use crate::error::Result;
use crate::upower::Monitor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct AppConfig {
  /// Force polling interval in seconds. If not set, the monitor will
  /// only poll when the power source or battery status changes.
  #[arg(short = 'i', long)]
  force_poll_interval: Option<u32>,

  /// Alert threshold in percent. If the battery percentage is below
  /// this threshold, the alert command will be executed.
  #[arg(short = 'l', long, default_value_t = 10)]
  alert_threshold: u32,

  /// Command to execute when the battery percentage is below the
  /// alert threshold. The command will be executed with `sh -c`.
  #[arg(short = 'c', long)]
  alert_command: Option<String>,

  /// Refire interval in seconds. The alert command will only be
  /// executed once every `alert_refire_interval` seconds as long as
  /// the battery percentage is below the alert threshold.
  #[arg(short = 'r', long)]
  alert_refire_interval: Option<u32>,
}

pub struct App {
  config: AppConfig,
  monitor: Monitor,
}

impl App {
  pub async fn new_from_env() -> Result<Self> {
    let monitor = Monitor::new().await?;
    let config = AppConfig::parse();

    Ok(Self { config, monitor })
  }

  pub async fn run(&self) -> Result<()> {
    select! {
      _ = self.handle_changes() => {},
      _ = self.monitor.run() => {},
    }

    Ok(())
  }

  pub async fn handle_changes(&self) -> Result<()> {
    let mut last_output = None;
    loop {
      let state = self.monitor.changed_state().await?;
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
}
