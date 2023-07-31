use std::time::Instant;

use clap::Parser;
use tokio::select;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::upower::Monitor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct AppConfig {
  /// Force polling interval in seconds. If not set, the monitor will
  /// only poll when the power source or battery status changes.
  #[arg(short = 'i', long)]
  force_poll_interval: Option<u64>,

  /// Alert threshold in percent. If the battery percentage is below
  /// this threshold, the alert command will be executed.
  #[arg(short = 'l', long, default_value_t = 10.0)]
  alert_threshold: f64,

  /// Command to execute when the battery percentage is below the
  /// alert threshold. The command will be executed with `sh -c`.
  #[arg(short = 'c', long)]
  alert_command: Option<String>,

  /// Refire interval in seconds. The alert command will only be
  /// executed once every `alert_refire_interval` seconds as long as
  /// the battery percentage is below the alert threshold.
  #[arg(short = 'r', long)]
  alert_refire_interval: Option<u64>,
}

pub struct App {
  config: AppConfig,
  monitor: Monitor,
  last_alerted_at: RwLock<Option<Instant>>,
}

impl App {
  pub async fn new_from_env() -> Result<Self> {
    let monitor = Monitor::new().await?;
    let config = AppConfig::parse();
    let last_alerted_at = RwLock::new(None);

    Ok(Self {
      config,
      monitor,
      last_alerted_at,
    })
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

      self.handle_alert(state.percentage).await?;
    }
  }

  pub async fn handle_alert(&self, current: f64) -> Result<()> {
    if current > self.config.alert_threshold {
      self.last_alerted_at.write().await.take();
      return Ok(());
    }

    let Some(alert_command) = self.config.alert_command.as_ref() else {
      return Ok(());
    };

    let last_alerted_at = self.last_alerted_at.read().await;
    let refire_interval = self.config.alert_refire_interval;

    let should_alert = match last_alerted_at.as_ref() {
      Some(last_alerted_at) => {
        let elapsed = last_alerted_at.elapsed();
        refire_interval.is_some_and(|i| elapsed.as_secs() >= i)
      }
      None => true,
    };

    drop(last_alerted_at);

    if should_alert {
      self.last_alerted_at.write().await.replace(Instant::now());

      let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(alert_command)
        .spawn()?;
      child.wait().await?;
    }

    Ok(())
  }
}
