mod endpoints;
mod ingress;
mod hubs;
mod webrtc_wrapper;

use std::sync::Arc;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use config::{Config, ConfigBuilder, File, FileFormat};
use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;


#[actix_web::main]
async fn main() -> std::io::Result<()>{
    let builder = Config::builder()
        .add_source(File::new("config.toml", FileFormat::Toml))
        .build().unwrap();

    let level: String = builder.get("log.level").unwrap();
    let log_level = match level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    env_logger::builder()
        .filter_level(log_level)
        .init();

    log::info!("Starting server");
    println!("Hello, world!");

    let hub = Arc::new(Hub::new());
    let api = Arc::new(WebRtcApi::new());

    endpoints::build(hub, api).await
}
