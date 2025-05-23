use std::{fs::File, path::Path};

use anyhow::{Result, bail};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

fn main() {
    if let Err(e) = setup_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
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
