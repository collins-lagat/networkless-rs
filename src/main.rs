mod app;
mod interfaces;
mod network;
mod trays;

use std::{fs::File, panic, path::Path};

use anyhow::{Result, bail};
use app::{Action, App, Event};
use fs2::FileExt;
use futures::StreamExt;
use log::{LevelFilter, error, info, warn};
use network::network_manager::NetworkManager;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tokio::{
    fs,
    sync::mpsc::{Sender, channel},
};
use trays::TrayManager;
use zbus::{Connection, Proxy};

pub const APP_ID: &str = "com.collinslagat.applets.networkless";
const LOCK_FILE: &str = "networkless.lock";
const LOG_FILE: &str = "networkless.log";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    panic::set_hook(Box::new(|info| {
        error!("Unhandled panic: {}", info);
    }));

    let runtime_dir = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            bail!("XDG_RUNTIME_DIR not set: {}", e);
        }
    };

    let lock_file_path = Path::new(&runtime_dir).join(LOCK_FILE);

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

    wait_for_session_bus_and_status_notifier().await?;

    let (event_tx, event_rx) = channel::<Event>(32);
    let (action_tx, action_rx) = channel::<Action>(32);

    let signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals, event_tx.clone()));

    let connection = Connection::system().await?;
    let network_manager = NetworkManager::new(connection).await?;

    let app = App::new(event_tx, action_tx, network_manager);

    let tray_manager = TrayManager::new(app.clone());

    app.send_event(Event::Init).await;

    app.run(event_rx, action_rx, tray_manager).await;

    info!("Cleaning up");

    if let Err(e) = fs::remove_file(lock_file_path).await {
        error!("Failed to remove lock: {}", e);
    }

    handle.close();
    signals_task.await?;

    Ok(())
}

async fn handle_signals(mut signals: Signals, tx: Sender<Event>) {
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

    let log_file_path = Path::new(&runtime_dir).join(LOG_FILE);

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

async fn wait_for_session_bus_and_status_notifier() -> Result<()> {
    // Give up after 20 attempts spanning 100ms each for a total of 2s
    let max_attempts = 20;

    for attempt in 0..max_attempts {
        match Connection::session().await {
            Ok(connection) => {
                let status_notifier = Proxy::new(
                    &connection,
                    "org.kde.StatusNotifierWatcher",
                    "/StatusNotifierWatcher",
                    "org.kde.StatusNotifierWatcher",
                )
                .await?;

                match status_notifier
                    .get_property::<bool>("IsStatusNotifierHostRegistered")
                    .await
                {
                    Ok(_) => {
                        info!(
                            "Status notifier is available! (attempt {}/{})",
                            attempt, max_attempts
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        warn!(
                            "Failed to get IsStatusNotifierHostRegistered: {} (attempt {}/{})",
                            e, attempt, max_attempts
                        );
                    }
                };
            }
            Err(e) => {
                warn!(
                    "Failed to connect to session bus: {} (attempt {}/{})",
                    e, attempt, max_attempts
                );
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    Err(anyhow::anyhow!("Failed to connect to session bus"))
}
