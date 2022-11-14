use std::io;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::{error, info};
use s3::Bucket;

use crate::config::{load_config, ServerConfig};

mod bucket;
mod build;
mod config;
mod logging;
mod package;
mod routes;
mod validate;

#[cfg(test)]
mod tests;

pub struct AppState {
    bucket: Bucket,
}

impl AppState {
    const fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }
}

async fn start_server(config: ServerConfig) -> io::Result<()> {
    HttpServer::new(move || {
        let bucket = bucket::load_bucket(&config.bucket_info).expect("Error while loading bucket");
        let mut cors = Cors::default();

        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(web::Data::new(AppState::new(bucket)))
            .service(routes::resource_version_amount)
            .wrap(cors)
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
