use std::io;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use log::{error, info};

use crate::config::{load_config, ConfigError, ServerConfig};

mod build;
mod config;
mod db;
mod logging;
mod package;
mod routes;

#[cfg(test)]
mod tests;

async fn start_server(config: ServerConfig) -> io::Result<()> {
    HttpServer::new(move || {
        let cors = match &config.cors_origin {
            Some(v) => Cors::default().allowed_origin(v),
            None => Cors::permissive(),
        };

        App::new().wrap(cors)
    })
    .bind(config.bind_addr)?
    .run()
    .await
}

#[actix_web::main]
async fn main() {
    println!("Initializing logger...");

    if let Err(err) = logging::setup_logger() {
        eprintln!("Could not initialize logger: {err}");
        return;
    }

    info!("Starting server...");
    info!("Build type: {:?}", build::BUILD);

    info!("Loading config...");

    let config = match load_config() {
        Ok(v) => v,
        Err(err) => {
            match err {
                ConfigError::Io(err) => error!("An IO error occurred: {err}"),
                ConfigError::Toml(err) => {
                    error!("An error occurred while parsing the config: {err}");
                }
            }
            return;
        }
    };

    info!("Loaded configuration:\n{config:#?}");

    if let Err(err) = start_server(config).await {
        error!("An error occurred, causing the server to stop: {err}");
    } else {
        info!("The server returned without an error.");
    }
}
