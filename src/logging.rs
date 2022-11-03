use std::{fs, io};

use chrono::Local;
use fern::{Dispatch, InitError};
use log::LevelFilter;

use crate::build;
use crate::build::Build;

pub fn setup_logger() -> Result<(), InitError> {
    fs::create_dir_all("logs")?;

    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("[%Y-%m-%d] [%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(match build::BUILD {
            Build::Development => LevelFilter::Debug,
            Build::Production => LevelFilter::Info,
        })
        .chain(io::stdout())
        .chain(fern::log_file(
            Local::now()
                .format("logs/log-%Y_%m_%d_%H-%M-%S.txt")
                .to_string(),
        )?)
        .apply()?;
    Ok(())
}
