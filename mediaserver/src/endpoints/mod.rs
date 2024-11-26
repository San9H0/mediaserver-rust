use crate::hubs::hub::Hub;
use crate::{egress, ingress};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

pub mod error;
mod file;
mod hls;
pub mod whep;
pub mod whip;

pub async fn build(hub: Arc<Hub>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Container {
                whip_server: ingress::servers::whip::WhipServer::new(hub.clone()),
                whep_server: egress::servers::whep::WhepServer::new(hub.clone()),
                record_server: egress::servers::record::RecordServer::new(hub.clone()),
                hls_server: egress::servers::hls::HlsServer::new(hub.clone()),
            }))
            .wrap(Logger::default())
            .configure(routes)
    })
    .workers(1)
    .bind("0.0.0.0:9090")? // 바인딩이 실패하면 `?`로 에러가 전파됨
    .run()
    .await
}

fn routes(app: &mut web::ServiceConfig) {
    app.service(web::resource("/v1/whip").route(web::post().to(whip::handle_whip)))
        .service(web::resource("/v1/whep").route(web::post().to(whep::handle_whep)))
        .service(web::resource("/v1/record").route(web::post().to(file::handle_create_record)))
        .service(web::resource("/v1/hls").route(web::post().to(hls::handle_create_session)))
        .service(
            web::resource("/v1/record/{session_id}")
                .route(web::delete().to(file::handle_delete_record)),
        )
        .service(
            web::resource("/v1/hls/{session_id}")
                .route(web::delete().to(hls::handle_delete_session))
                .route(web::get().to(hls::handle_get_session)),
        )
        .service(
            web::resource("/v1/public/{session_id}/hls/{hls}").route(web::get().to(
                |handler: web::Data<Container>, path, query| {
                    hls::handle_get_hls(handler, "hls", path, query)
                },
            )),
        )
        .service(
            web::resource("/v1/public/{session_id}/llhls/{hls}").route(web::get().to(
                |handler: web::Data<Container>, path, query| {
                    hls::handle_get_hls(handler, "llhls", path, query)
                },
            )),
        );
}

pub struct Container {
    pub whip_server: Arc<ingress::servers::whip::WhipServer>,
    pub whep_server: Arc<egress::servers::whep::WhepServer>,
    pub record_server: Arc<egress::servers::record::RecordServer>,
    pub hls_server: Arc<egress::servers::hls::HlsServer>,
}
