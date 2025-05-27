mod app;
mod dbus;
mod event;
mod icon;

use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use event::Event;
use fs2::FileExt;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use icon::TrayIcon;
use log::{LevelFilter, info};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = acquire_lock() {
        eprintln!("Failed to acquire lock: {}", e);
        std::process::exit(1);
    }

    info!("Lock acquired");

    let (mut tx, mut rx) = channel::<Event>(32);

    let signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals, tx.clone()));

    let mut tray_icon = TrayIcon::new();

    tx.send(Event::Init).await?;

    loop {
        match rx.next().await {
            Some(event) => match event {
                Event::Init => {
                    info!("Initializing");
                }
                Event::WifiEnabled(enabled) => {
                    info!("Wifi enabled: {}", enabled);
                    tray_icon.send(event).await;
                }
                Event::AirplaneMode(enabled) => {
                    info!("Airplane mode: {}", enabled);
                    tray_icon.send(event).await;
                }
                Event::Shutdown => {
                    info!("Shutting down");
                    break;
                }
            },
            None => {
                info!("No event");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    handle.close();
    signals_task.await?;

    Ok(())
}

async fn handle_signals(mut signals: Signals, mut tx: Sender<Event>) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT => {
                info!("Received signal {}", signal);
                let _ = tx.send(Event::Shutdown).await;
            }
            _ => unreachable!(),
        }
    }
}

fn setup_logging() -> Result<()> {
    let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            bail!("XDG_RUNTIME_DIR not set: {}", e);
        }
    };

    let log_file_path = Path::new(&runtime_dir).join("networkless.log");

    let log_file = match File::create(log_file_path) {
        Ok(file) => file,
        Err(e) => {
            bail!("Failed to create log file: {}", e);
        }
    };

    if let Err(e) = CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
    ]) {
        bail!("Failed to initialize logging: {}", e);
    };

    Ok(())
}

fn acquire_lock() -> Result<()> {
    let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            bail!("XDG_RUNTIME_DIR not set: {}", e);
        }
    };

    let lock_file_path = Path::new(&runtime_dir).join("networkless.lock");

    let lock_file = match File::create(lock_file_path) {
        Ok(file) => file,
        Err(e) => {
            bail!("Failed to create lock file: {}", e);
        }
    };

    if let Err(e) = lock_file.lock_exclusive() {
        bail!("Failed to acquire lock: {}", e);
    }

    Ok(())
}
