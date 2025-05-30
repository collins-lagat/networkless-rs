mod event;
mod icon;
mod interfaces;
mod nm;

use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use event::Event;
use fs2::FileExt;
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{Sender, channel},
};
use icon::TrayIcon;
use interfaces::{access_point::AccessPointProxy, wireless::WirelessProxy};
use log::{LevelFilter, error, info};
use nm::{Connectivity, DeviceType, NetworkManager, State as NmState};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tokio::fs;
use zbus::Connection;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            bail!("XDG_RUNTIME_DIR not set: {}", e);
        }
    };

    let lock_file_path = Path::new(&runtime_dir).join("networkless.lock");

    let lock_file = match File::create(&lock_file_path) {
        Ok(file) => file,
        Err(e) => {
            bail!("Failed to create lock file: {}", e);
        }
    };

    if lock_file.try_lock_exclusive().is_err() {
        bail!("Failed to acquire lock. Another instance is running.");
    }

    info!("Lock acquired");

    let (mut tx, mut rx) = channel::<Event>(32);

    let signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals, tx.clone()));

    let mut tray_icon = TrayIcon::new();

    let nm = NetworkManager::new().await?;

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
                NmState::Asleep | NmState::ConnectedLocal | NmState::ConnectedSite => {
                    if _nm.airplane_mode_enabled().await.unwrap() {
                        Event::AirplaneMode
                    } else {
                        Event::Off
                    }
                }
                NmState::Disconnected => {
                    if _nm.airplane_mode_enabled().await.unwrap() {
                        Event::AirplaneMode
                    } else {
                        Event::Disconnected
                    }
                }
                NmState::Connecting | NmState::Disconnecting => Event::Busy,
                NmState::ConnectedGlobal => match _nm.connectivity().await.unwrap() {
                    Connectivity::Full => {
                        let connection = _nm.active_connection().await.unwrap();

                        let device_type = connection
                            .devices()
                            .await
                            .unwrap()
                            .first()
                            .unwrap()
                            .device_type()
                            .await
                            .unwrap();

                        match device_type {
                            DeviceType::Wifi => {
                                let conn = Connection::system().await.unwrap();
                                let device_path = connection
                                    .devices()
                                    .await
                                    .unwrap()
                                    .first()
                                    .unwrap()
                                    .path
                                    .clone();
                                let wireless =
                                    WirelessProxy::new(&conn, device_path).await.unwrap();
                                let active_access_point = AccessPointProxy::new(
                                    &conn,
                                    wireless.active_access_point().await.unwrap(),
                                )
                                .await
                                .unwrap();

                                let strength = active_access_point.strength().await.unwrap();

                                Event::Wifi(strength)
                            }
                            DeviceType::TunTap => Event::Vpn,
                            DeviceType::Ethernet => Event::Ethernet,
                            _ => Event::Unknown,
                        }
                    }
                    Connectivity::Loss => Event::Limited,
                    _ => Event::Limited,
                },
                NmState::Unknown => Event::Unknown,
            };

            if let Err(e) = tx.send(event).await {
                error!("Failed to send event: {}", e);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });

    loop {
        match rx.next().await {
            Some(event) => {
                let _event = event.clone();
                tray_icon.send(_event).await;
                if let Event::Shutdown = event {
                    break;
                }
            }
            None => {
                info!("No event");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    if let Err(e) = fs::remove_file(lock_file_path).await {
        error!("Failed to remove lock: {}", e);
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
