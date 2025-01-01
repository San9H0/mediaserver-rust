use crate::hubs::hub::Hub;
use crate::{egress, ingress};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{from_fn, Logger};
use actix_web::Error;
use actix_web::{web, App, HttpRequest, HttpServer};
use std::sync::Arc;

pub mod error;
mod hls;
pub mod whep;
pub mod whip;

pub async fn build(hub: Arc<Hub>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Container {
                whip_server: ingress::servers::whip::WhipServer::new(hub.clone()),
                whep_server: egress::servers::whep::WhepServer::new(hub.clone()),
                hls_server: egress::servers::hls::HlsServer::new(hub.clone()),
            }))
            .wrap(from_fn(my_middleware))
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
        .service(web::resource("/v1/hls").route(web::post().to(hls::handle_create_session)))
        .service(
            web::resource("/v1/hls/{session_id}")
                .route(web::delete().to(hls::handle_delete_session))
                .route(web::get().to(hls::handle_get_session)),
        )
        .service(
            web::resource("/v1/public/hls/{session_id}/{filename:.*}").route(web::get().to(
                |req: HttpRequest, handler: web::Data<Container>, path, query| {
                    hls::handle_get_hls(req, handler, path, query)
                },
            )),
        );
}

pub struct Container {
    pub whip_server: Arc<ingress::servers::whip::WhipServer>,
    pub whep_server: Arc<egress::servers::whep::WhepServer>,
    pub hls_server: Arc<egress::servers::hls::HlsServer>,
}

async fn my_middleware(
    req: ServiceRequest,
    next: actix_web::middleware::Next<impl actix_web::body::MessageBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    let method = req.method().clone();
    let path = req.path().to_owned();
    log::info!("incoming request method:{:?}, path:{:?}", method, path);
    let resp = next.call(req).await;
    if let Ok(ref resp) = resp {
        log::info!(
            "response method:{:?}, path:{:?}, status:{:?}",
            method,
            path,
            resp.status()
        );
    } else {
        log::error!("resp error");
    }
    return resp;
}
