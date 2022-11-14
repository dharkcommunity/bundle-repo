use std::fmt::{Debug, Formatter};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::{fs, io};

use cnsl::readln;
use log::{info, warn};
use s3::Region;
use serde::{Deserialize, Serialize};

use crate::build;
use crate::build::Build;

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key: String,
    pub secret_key: String,
}

impl Credentials {
    const fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
            secret_key,
        }
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Credentials hidden]")
    }
}

// Region::Custom is neither serializable or deserializable, so we make a wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionWrapper {
    region: String,
    endpoint: String,
}

impl RegionWrapper {
    pub fn into_region(self) -> Region {
        Region::Custom {
            region: self.region,
            endpoint: self.endpoint,
        }
    }

    const fn new(region: String, endpoint: String) -> Self {
        Self { region, endpoint }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub name: String,
    pub region: RegionWrapper,
    pub credentials: Credentials,
}

impl BucketInfo {
    const fn new(name: String, region: RegionWrapper, credentials: Credentials) -> Self {
        Self {
            name,
            region,
            credentials,
        }
    }
}

/// A serializable struct containing adjustable server settings.
///
/// Configurations are loaded from the `<working_dir>/config/` folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub cors_origins: Vec<String>,
    pub bucket_info: BucketInfo,
}

impl ServerConfig {
    const fn new(
        bind_addr: SocketAddr,
        cors_origins: Vec<String>,
        bucket_info: BucketInfo,
    ) -> Self {
        Self {
            bind_addr,
            cors_origins,
            bucket_info,
        }
    }
}

/// If the user has not yet defined the required [BucketInfo], they will not be met with an
/// error. Instead, they will be prompted through **stdin** for said information through the
/// [ServerConfigCliWrapper::into_server_config_prompted] function.
#[derive(Deserialize)]
struct ServerConfigCliWrapper {
    bind_addr: SocketAddr,
    cors_origins: Vec<String>,
    bucket_info: Option<BucketInfo>,
}

impl ServerConfigCliWrapper {
    fn default_development() -> Self {
        Self::new("127.0.0.1:8080".parse().unwrap(), Vec::new())
    }

    fn default_production() -> Self {
        Self::new("0.0.0.0:8080".parse().unwrap(), Vec::new())
    }

    fn default_by_build() -> Self {
        match build::BUILD {
            Build::Development => Self::default_development(),
            Build::Production => Self::default_production(),
        }
    }

    fn into_server_config_prompted(self) -> ServerConfig {
        let bucket_info = self.bucket_info.unwrap_or_else(|| {
            warn!(":= The following values are missing in your loaded configuration:");
            info!(" - bucket region");
            info!(" - bucket endpoint");
            info!(" - bucket name");
            info!(" - bucket access key");
            info!(" - bucket secret key");
            info!("Enter the missing values below:");

            let region = RegionWrapper::new(
                readln!(" >> bucket region: "),
                readln!(" >> bucket endpoint: "),
            );

            let info = BucketInfo::new(
                readln!(" >> bucket name: "),
                region,
                Credentials::new(
                    rpassword::prompt_password(" >> bucket access key: ").unwrap(),
                    rpassword::prompt_password(" >> bucket secret key: ").unwrap(),
                ),
            );

            info!("Accepted bucket information: {info:#?}");
            info
        });
        ServerConfig::new(self.bind_addr, self.cors_origins, bucket_info)
    }

    const fn new(bind_addr: SocketAddr, cors_origins: Vec<String>) -> Self {
        Self {
            bind_addr,
            cors_origins,
            bucket_info: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("An IO error occurred: {0}")]
    Io(#[from] io::Error),

    #[error("An error occurred while parsing the configuration: {0}")]
    Toml(#[from] toml::de::Error),
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
