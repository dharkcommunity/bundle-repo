use std::io;

use actix_cors::Cors;
use actix_web::{App, HttpServer};
use log::{error, info};

use crate::config::{load_config, ServerConfig};

mod bucket;
mod build;
mod config;
mod logging;
mod package;
mod routes;

#[cfg(test)]
mod tests;

async fn start_server(config: ServerConfig) -> io::Result<()> {
    HttpServer::new(move || {
        let mut cors = Cors::default();

        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }

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

    info!("Build type: {:?}", build::BUILD);
    info!("Loading config...");

    let config = match load_config() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("{err}");
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
