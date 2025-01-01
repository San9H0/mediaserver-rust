mod codecs;
mod egress;
mod endpoints;
mod hubs;
mod ingress;
mod protocols;
mod utils;
mod webrtc_wrapper;

use crate::hubs::hub::Hub;
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

    let mut logger_builder = flexi_logger::Logger::try_with_str(&level)
        .unwrap()
        .write_mode(flexi_logger::WriteMode::BufferAndFlush);

    if config.get("log.stdout").unwrap_or(false) {
        logger_builder = logger_builder.log_to_stdout();
    } else if config.get("log.file").unwrap_or(false) {
        logger_builder = logger_builder.log_to_file(flexi_logger::FileSpec::default());
    }

    logger_builder.start().unwrap();
}
