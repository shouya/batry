## Battery status monitor

This is a small program that monitors battery status through
[UPower](https://upower.freedesktop.org/)'s DBus interface and outputs
status updates as JSON.

I use it to serve battery information for the battery widget on my
status bar.

### Reported information

- battery percentage
- wattage information
- status (charging, discharging, fully_charged, not_charging, unknown)
- time to full
- time to empty

### Features

- Notification-based: The program is notified of changes, not polling for them.
  + Optionally, you can specify a force polling interval (`--min-poll-interval`) if you want.
- Alert: Run a custom alert command on low battery.
- Repeated alert: Run the alert command every X seconds if the battery remains low.
- Lightweight: The program is carefully crafted and uses negligible resources.

### Usage

``` text
$ batry --help
Usage: batry [OPTIONS]

Options:
  -i, --min-poll-interval <MIN_POLL_INTERVAL>
          Minimal polling interval in seconds. If not set, the monitor will only poll when the power source or battery status changes.
  -t, --alert-threshold <ALERT_THRESHOLD>
          Alert threshold in percent. If the battery percentage is below this threshold, the alert command will be executed [default: 10].
  -c, --alert-command <ALERT_COMMAND>
          Command to execute when the battery percentage is below the alert threshold. The command will be executed with `sh -c`.
  -r, --alert-refire-interval <ALERT_REFIRE_INTERVAL>
          Refire interval in seconds. The alert command will only be executed once every `alert_refire_interval` seconds as long as the battery percentage is below the alert threshold.
  -h, --help
          Print help.
  -V, --version
          Print version.

```

The output in each line is in JSON format.

### Build

You can run `cargo build --release` to build the program.
