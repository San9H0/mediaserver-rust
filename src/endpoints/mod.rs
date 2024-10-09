use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use crate::{endpoints, ingress};
use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;

pub mod whip;
pub mod whep;

pub async fn build(hub : Arc<Hub>, api: Arc<WebRtcApi>) -> std::io::Result<()> {
    HttpServer::new(move|| {
        App::new()
            .app_data(web::Data::new(Container{
                whip_server: ingress::servers::whip::WhipServer::new(hub.clone(), api.clone()),
            }))
            .wrap(Logger::default())
            .configure(routes)
    })
        .bind("0.0.0.0:9090")?  // 바인딩이 실패하면 `?`로 에러가 전파됨
        .run()
        .await
}

fn routes(app: &mut web::ServiceConfig) {
    app.service(web::resource("/v1/whip").route(web::post().to(endpoints::whip::handle_whip)))
        .service(web::resource("/v1/whep").route(web::post().to(endpoints::whep::handle_whep)));
}

pub struct Container {
    pub whip_server: ingress::servers::whip::WhipServer,
}