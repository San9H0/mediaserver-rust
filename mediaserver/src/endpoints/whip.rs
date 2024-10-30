use crate::endpoints::Container;
use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn handle_whip(
    handler: web::Data<Container>,
    offer: String,
    auth: BearerAuth,
) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!("whip stream_id:{}, offer:{}", token, offer);

    let result = handler.whip_server.start_session(token, &offer).await;

    let answer = match result {
        Ok(answer) => answer,
        Err(e) => {
            log::error!("whip error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("whip answer:{}", answer);

    HttpResponse::Created()
        .insert_header(("Content-Type", "application/sdp"))
        .insert_header(("Location", "http://127.0.0.1/v1/whip/candidates"))
        .body(answer)
}
