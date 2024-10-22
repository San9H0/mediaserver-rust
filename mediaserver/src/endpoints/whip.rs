use crate::endpoints::Container;
use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[derive(Clone)]
pub struct WhipHandler {}

impl WhipHandler {
    pub fn new() -> Self {
        WhipHandler {}
    }
}

pub async fn handle_whip(
    handler: web::Data<Container>,
    offer: String,
    auth: BearerAuth,
) -> impl Responder {
    let token = auth.token().to_owned();

    let answer = match handler
        .whip_server
        .start_session(token.to_owned(), offer.to_string())
        .await
    {
        Ok(answer) => answer,
        Err(e) => {
            log::error!("whip error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Created()
        .insert_header(("Content-Type", "application/sdp"))
        .insert_header(("Location", "http://127.0.0.1/v1/whip/candidates"))
        .body(answer)
}
