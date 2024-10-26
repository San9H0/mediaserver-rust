mod codecs;
mod egress;
mod endpoints;
mod hubs;
mod ingress;
mod webrtc_wrapper;

mod utils;

use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use config::{Config, File, FileFormat};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Arc::new(
        Config::builder()
            .add_source(File::new("config.toml", FileFormat::Toml))
            .build()
            .unwrap(),
    );

    init_log(config.clone());

    let num_workers = config
        .get("general.workers")
        .map(|w| if w == 0 { num_cpus::get() } else { w })
        .unwrap_or_else(|_| num_cpus::get());

    log::info!("Starting server");
    println!("Hello, world!");

    let _tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_workers) // 스레드 수를 8개로 설정
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    let hub = Hub::new();
    endpoints::build(hub.clone()).await
}

fn init_log(config: Arc<Config>) {
    let level: String = config.get("log.level").unwrap();
    let log_level = match level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    env_logger::builder().filter_level(log_level).init();
}