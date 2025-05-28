mod app;
mod dbus;
mod event;
mod icon;
mod nm;

use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use app::App;
use event::Event;
use fs2::FileExt;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use icon::TrayIcon;
use log::{LevelFilter, error, info};
use nm::{NetworkManager, State as NmState};
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

    let nm = NetworkManager::new().await?;

    let app = App::new(&nm).await;

    let _nm = nm.clone();
    tokio::spawn(async move {
        loop {
            let state = match _nm.state().await {
                Ok(state) => state,
                Err(e) => {
                    error!("Failed to get state: {}", e);
                    continue;
                }
            };

            let event = match state {
                NmState::Asleep => Event::Off,
                NmState::Disconnected => Event::Disconnected,
                NmState::Connecting | NmState::Disconnecting => Event::Busy,
                NmState::ConnectedGlobal | NmState::ConnectedSite | NmState::ConnectedLocal => {
                    Event::Wifi(0)
                }
                NmState::Unknown => Event::Unknown,
            };

            if let Err(e) = tx.send(event).await {
                error!("Failed to send event: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    loop {
        match rx.next().await {
            Some(event) => {
                let _event = event.clone();
                tray_icon.send(_event).await;
                match event {
                    Event::Unknown => {}
                    Event::Off => {}
                    Event::Busy => {}
                    Event::Disconnected => {}
                    Event::AirplaneMode => {}
                    Event::Limited => {}
                    Event::VPN => {}
                    Event::Ethernet => {}
                    Event::Wifi(strength) => {
                        info!("Wifi strength: {}", strength);
                    }
                    Event::Shutdown => {
                        info!("Shutting down");
                        break;
                    }
                }
            }
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
