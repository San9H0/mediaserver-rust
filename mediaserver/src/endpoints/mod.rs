use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use crate::{egress, ingress};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;

pub mod error;
mod file;
pub mod whep;
pub mod whip;

pub async fn myf(failed: bool) -> error::Result<impl Responder> {
    if failed {
        // return anyhow::anyhow!("failed");
        return Err(error::Error::from(anyhow::anyhow!("failed")));
    }
    Ok(HttpResponse::Ok())
}

pub async fn build(hub: Arc<Hub>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Container {
                whip_server: ingress::servers::whip::WhipServer::new(hub.clone()),
                whep_server: egress::servers::whep::WhepServer::new(hub.clone()),
                record_server: egress::servers::record::RecordServer::new(hub.clone()),
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
        .service(web::resource("/v1/record").route(web::post().to(file::handle_record)));
}

pub struct Container {
    pub whip_server: Arc<ingress::servers::whip::WhipServer>,
    pub whep_server: Arc<egress::servers::whep::WhepServer>,
    pub record_server: Arc<egress::servers::record::RecordServer>,
}
