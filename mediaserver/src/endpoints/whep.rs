use crate::endpoints::Container;
use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn handle_whep(
    handler: web::Data<Container>,
    offer: String,
    auth: BearerAuth,
) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!(
        "whep body streamID:{}, messageType:request, body:{}",
        token,
        offer
    );

    let answer = match handler
        .whep_server
        .start_session(token.to_owned(), &offer)
        .await
    {
        Ok(answer) => answer,
        Err(e) => {
            log::error!("whep error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("whep response streamID:{}, body:{}", token, answer);

    HttpResponse::Created()
        .insert_header(("Content-Type", "application/sdp"))
        .insert_header(("Location", "http://127.0.0.1/v1/whip/candidates"))
        .body(answer)
}
