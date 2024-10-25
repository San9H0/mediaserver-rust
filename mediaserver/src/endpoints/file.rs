use crate::endpoints::Container;
use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Serialize;

#[derive(Clone)]
pub struct FileRecordHandler {}

impl FileRecordHandler {
    pub fn new() -> Self {
        FileRecordHandler {}
    }
}

#[derive(Serialize)]
struct CreateRecordResponse {
    session_id: String,
}

pub async fn handle_create_record(handler: web::Data<Container>, auth: BearerAuth) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!("record file body streamID:{}, messageType:request", token,);

    let session_id = match handler.record_server.start_session(&token).await {
        Ok(session_id) => session_id,
        Err(e) => {
            log::error!("record file error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("record file response streamID:{}", &token);

    let response = CreateRecordResponse{ session_id };

    HttpResponse::Ok().json(response)
}

pub async fn handle_delete_record(handler: web::Data<Container>, session_id_: web::Path<String>) -> impl Responder {
    let session_id = session_id_.to_string();

    log::info!("delete record file body streamID:{}, messageType:request", session_id,);

    let _result = match handler.record_server.stop_session(session_id.to_owned()).await {
        Ok(_result) => _result,
        Err(e) => {
            log::error!("delete record file error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().finish()
}