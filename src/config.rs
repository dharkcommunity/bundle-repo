use crate::build;
use crate::build::Build;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::net::SocketAddr;

use std::path::Path;
use std::{fs, io};

/// A serializable struct containing adjustable server settings.
///
/// Configurations are loaded from the `<working_dir>/config/` folder.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub cors_origin: Option<String>,
}

impl ServerConfig {
    const fn new(bind_addr: SocketAddr, cors_origin: Option<String>) -> Self {
        Self {
            bind_addr,
            cors_origin,
        }
    }

    fn default_dev() -> Self {
        let bind_addr = "127.0.0.1:8080".parse().unwrap();
        Self::new(bind_addr, None)
    }

    fn default_production() -> Self {
        Self::new(
            "0.0.0.0:8080".parse().unwrap(),
            Some("https://imajindevon.com".to_string()),
        )
    }
}

pub enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::Toml(err)
    }
}

pub type Result = std::result::Result<ServerConfig, ConfigError>;

fn load_or_init<P, F>(path: P, init: F) -> Result
where
    P: AsRef<Path>,
    F: FnOnce() -> ServerConfig,
{
    let content = fs::read_to_string(&path);

    if let Err(err) = &content {
        if err.kind() == ErrorKind::NotFound {
            let data = init();

            fs::create_dir_all("config")?;
            fs::write(path, toml::to_string(&data).unwrap())?;

            return Ok(data);
        }
    }
    toml::from_str(&content?).map_err(ConfigError::Toml)
}

/// Try to deserialize the TOML file at the given path into a [ServerConfig].
///
/// This function will return different configurations depending on the type of [Build].
pub fn load_config() -> Result {
    match build::BUILD {
        Build::Development => load_or_init("config/Development.toml", ServerConfig::default_dev),
        Build::Production => {
            load_or_init("config/Production.toml", ServerConfig::default_production)
        }
    }
}
