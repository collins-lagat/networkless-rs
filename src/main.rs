mod app;
mod interfaces;
mod network;
mod tray;

use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use app::{Action, App, Event};
use fs2::FileExt;
use futures::StreamExt;
use ksni::TrayMethods;
use log::{LevelFilter, error, info};
use network::network_manager::NetworkManager;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tokio::{
    fs,
    sync::mpsc::{Sender, channel},
};
use tray::Tray;
use zbus::Connection;

pub const APP_ID: &str = "com.collinslagat.applets.networkless";
const LOCK_FILE: &str = "networkless.lock";
const LOG_FILE: &str = "networkless.log";

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

    let (event_tx, event_rx) = channel::<Event>(32);
    let (action_tx, action_rx) = channel::<Action>(32);

    let signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    let handle = signals.handle();

    let signals_task = tokio::spawn(handle_signals(signals, event_tx.clone()));

    let mut tray = Tray::new();

    let connection = Connection::system().await?;
    let network_manager = NetworkManager::new(connection).await?;

    let app = App::new(event_tx, action_tx, network_manager);

    tray.set_app(app.clone());

    let tray_handle = tray.spawn().await?;

    app.run(event_rx, action_rx, tray_handle).await;

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
