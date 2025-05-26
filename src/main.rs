mod app;
mod event;

use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use event::Event;
use fs2::FileExt;
use log::{LevelFilter, info};
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};
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

    let mut app = app::App::default();

    let mut signals = Signals::new([SIGINT, SIGTERM])?;

    let signal_tx = app.events.sender();
    tokio::spawn(async move {
        for signal in signals.forever() {
            info!("Received signal {:?}", signal);
            signal_tx.send(Event::Shutdown).unwrap();
        }
    });

    app.events.send(Event::Init).await;

    loop {
        match app.events.next().await {
            Some(event) => match event {
                Event::Init => {
                    info!("Initializing");
                }
                Event::WifiEnabled(enabled) => {
                    info!("Wifi enabled: {}", enabled);
                    app.wifi_enabled = enabled;
                }
                Event::AirplaneMode(enabled) => {
                    info!("Airplane mode: {}", enabled);
                    app.airplane_mode = enabled;
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

    Ok(())
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
