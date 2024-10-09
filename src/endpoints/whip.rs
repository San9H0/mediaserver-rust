use actix_web::{web, HttpResponse, Responder};
use actix_web::http::header::TryIntoHeaderPair;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::endpoints::Container;

#[derive(Clone)]
pub struct WhipHandler {
}

impl WhipHandler {
    pub fn new() -> Self {
        WhipHandler {  }
    }
}

pub async fn handle_whip(handler : web::Data<Container>, offer: String, auth: BearerAuth) -> impl Responder {
    let token = auth.token().to_owned();
    let answer = handler.whip_server.start_session(token.to_owned(), offer.to_string()).await.unwrap();
    log::info!(
        "whip body streamID:{}, messageType:request, body:{}",
        token, offer
    );

    HttpResponse::Created()
        .insert_header(("Content-Type", "application/sdp"))
        .insert_header(("Location", "http://127.0.0.1/v1/whip/candidates"))
        .body(answer)
}
