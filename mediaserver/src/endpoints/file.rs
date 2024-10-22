use crate::endpoints::Container;
use actix_web::{web, HttpResponse, Responder};
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

    let _result = match handler.record_server.start_session(token.to_owned()).await {
        Ok(_result) => _result,
        Err(e) => {
            log::error!("record file error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("record file response streamID:{}", token);

    HttpResponse::Ok().finish()
}
