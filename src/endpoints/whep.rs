use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::endpoints::Container;

#[derive(Clone)]
pub struct WhepHandler {
    pub args: String,
}

impl WhepHandler {
    pub fn new(args: String) -> Self {
        WhepHandler { args }
    }
}

pub async fn handle_whep(handler : web::Data<Container>, offer: String, auth: BearerAuth) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!(
        "whep body streamID:{}, messageType:request, body:{}",
        token, offer
    );

    let answer = handler.whep_server.start_session(token.to_owned(), offer.to_string()).await.unwrap();

    log::info!(
        "whep response streamID:{}, body:{}",
        token, answer
    );

    HttpResponse::Created()
        .insert_header(("Content-Type", "application/sdp"))
        .insert_header(("Location", "http://127.0.0.1/v1/whip/candidates"))
        .body(answer)
}
