use crate::endpoints::Container;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[derive(Clone)]
pub struct FileRecordHandler {}

impl FileRecordHandler {
    pub fn new() -> Self {
        FileRecordHandler {}
    }
}

pub async fn handle_record(handler: web::Data<Container>, auth: BearerAuth) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!("record file body streamID:{}, messageType:request", token,);

    handler
        .record_file_server
        .start_session(token.to_owned())
        .await
        .unwrap();

    log::info!("record file response streamID:{}", token);

    HttpResponse::Ok()
}
