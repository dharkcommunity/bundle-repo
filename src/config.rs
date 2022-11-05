use std::io::ErrorKind;
use std::net::SocketAddr;
use std::{fs, io};

use cnsl::readln;
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::build;
use crate::build::Build;

#[derive(Debug, Serialize, Deserialize)]
pub struct DbInformation {
    connection_url: String,
    cert_path: String,
}

impl DbInformation {
    #[allow(unused)]
    pub fn connection_url(&self) -> &str {
        &self.connection_url
    }

    #[allow(unused)]
    pub fn cert_path(&self) -> &str {
        &self.cert_path
    }

    const fn new(connection_url: String, cert_path: String) -> Self {
        Self {
            connection_url,
            cert_path,
        }
    }
}

/// A serializable struct containing adjustable server settings.
///
/// Configurations are loaded from the `<working_dir>/config/` folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub cors_origins: Option<Vec<String>>,
    pub database_info: DbInformation,
}

impl ServerConfig {
    const fn new(
        bind_addr: SocketAddr,
        cors_origins: Option<Vec<String>>,
        database_info: DbInformation,
    ) -> Self {
        Self {
            bind_addr,
            cors_origins,
            database_info,
        }
    }
}

/// If the user has not yet defined the required [DbInformation], they will not be met with an
/// error. Instead, they will be prompted through **stdin** for said information through the
/// [ServerConfigCliWrapper::into_server_config_prompted] function.
#[derive(Deserialize)]
struct ServerConfigCliWrapper {
    bind_addr: SocketAddr,
    cors_origins: Option<Vec<String>>,
    database_info: Option<DbInformation>,
}

impl ServerConfigCliWrapper {
    fn default_development() -> Self {
        Self::new("127.0.0.1:8080".parse().unwrap(), None)
    }

    fn default_production() -> Self {
        Self::new("0.0.0.0:8080".parse().unwrap(), Some(Vec::new()))
    }

    fn default_by_build() -> Self {
        match build::BUILD {
            Build::Development => Self::default_development(),
            Build::Production => Self::default_production(),
        }
    }

    fn into_server_config_prompted(self) -> ServerConfig {
        let database_info = self.database_info.unwrap_or_else(|| {
            warn!("=: The following values are missing in your loaded configuration:");
            info!(" - Database connection URL");
            info!(" - Database certification path");
            info!("Enter the missing values below:");

            let info = DbInformation::new(
                readln!(" >> Database connection URL: "),
                readln!(" >> Database certification path (.key): "),
            );

            info!("Accepted database information: {info:#?}");
            info
        });
        ServerConfig::new(self.bind_addr, self.cors_origins, database_info)
    }

    fn new(bind_addr: SocketAddr, cors_origins: Option<Vec<String>>) -> Self {
        Self {
            bind_addr,
            cors_origins,
            database_info: None,
        }
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

/// Try to deserialize the TOML file at the given path into a [ServerConfig].
///
/// This function will return different configurations depending on the type of [Build].
pub fn load_config() -> Result {
    let path = match build::BUILD {
        Build::Development => "config/Development.toml",
        Build::Production => "config/Production.toml",
    };

    let config = match fs::read_to_string(path) {
        Ok(txt) => toml::from_str(&txt)?,
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                ServerConfigCliWrapper::default_by_build()
            } else {
                return Err(ConfigError::Io(err));
            }
        }
    };

    let config = config.into_server_config_prompted();

    info!("({path}) Successfully loaded configuration, saving values...");
    fs::write(path, toml::to_string(&config).unwrap())?;

    Ok(config)
}
