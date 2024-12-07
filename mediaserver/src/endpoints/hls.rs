use std::path::{Path, PathBuf};

use crate::{egress::services::hls::config::HlsConfig, endpoints::Container};
use actix_files::NamedFile;
use actix_web::{http, web, HttpRequest, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};

use super::file;

#[derive(Serialize)]
struct HlsResponse {
    #[serde(rename = "sessionId")]
    session_id: String,
}

pub async fn handle_create_session(
    handler: web::Data<Container>,
    auth: BearerAuth,
) -> impl Responder {
    let token = auth.token().to_owned();

    log::info!(
        "hls_server file body streamID:{}, messageType:request",
        token,
    );

    let session_id = match handler.hls_server.start_session(&token).await {
        Ok(session_id) => session_id,
        Err(e) => {
            log::error!("hls error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    log::info!("hls_server file response streamID:{}", &token);

    let response = HlsResponse { session_id };

    HttpResponse::Ok().json(response)
}

pub async fn handle_get_session(
    _handler: web::Data<Container>,
    session_id_: web::Path<String>,
) -> impl Responder {
    let session_id = session_id_.to_string();

    log::info!("get hls body streamID:{}, messageType:request", session_id,);

    HttpResponse::Ok().finish()
}

pub async fn handle_delete_session(
    handler: web::Data<Container>,
    session_id_: web::Path<String>,
) -> impl Responder {
    let session_id = session_id_.to_string();

    log::info!(
        "delete hls file body streamID:{}, messageType:request",
        session_id,
    );

    let _result = match handler.hls_server.stop_session(session_id.to_owned()).await {
        Ok(_result) => _result,
        Err(e) => {
            log::error!("delete hls error:{}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct HlsQuery {
    #[serde(rename = "_HLS_msn")]
    pub _hls_msn: Option<u32>,
    #[serde(rename = "_HLS_part")]
    pub _hls_part: Option<u32>,
}

pub async fn handle_get_hls(
    req: HttpRequest,
    handler: web::Data<Container>,
    path: web::Path<(String, String)>,
    query: web::Query<HlsQuery>,
) -> actix_web::Result<HttpResponse> {
    let (session_id, filename) = path.into_inner();

    log::info!(
        "get hls body streamID:{}, filename:{} messageType:request",
        session_id,
        filename
    );
    let query = query.into_inner();

    let _session = match handler.hls_server.get_session(&session_id).await {
        Ok(session) => session,
        Err(err) => {
            log::error!("get hls error:{}", err);
            return Err(actix_web::error::ErrorNotFound("Session not found"));
        }
    };

    let hls_path = _session.config.clone();
    let Ok(filepath) = hls_path.get_path(&filename) else {
        log::error!("failed to get path");
        return Err(actix_web::error::ErrorNotFound("File not found"));
    };

    if let (Some(msn), Some(part)) = (query._hls_msn, query._hls_part) {
        // 두 값이 모두 있는 경우 처리
        let mut rx = _session.service.subscribe_signal();
        if rx.changed().await.is_ok() {
            let (recv_msn, recv_part) = *rx.borrow();
        }
    };

    if !filename.ends_with(".m3u8") && !filename.ends_with(".mp4") && !filename.ends_with(".m4s") {
        println!("Bad request");
        return Err(actix_web::error::ErrorBadRequest("Bad request"));
    }

    if filename.ends_with(".m3u8") {
        let v = tokio::fs::read_to_string(filepath.to_string()).await?;
        log::info!("get hls playlist : {}", v);
    } else if filename.ends_with(".mp4") || filename.ends_with(".m4s") {
        let v = tokio::fs::read(filepath.to_string()).await?;
        log::info!("get hls media : {:?}", v.len());
    }
    // if is_playlist {
    //     let v = tokio::fs::read_to_string(filepath.to_string()).await?;
    //     log::info!("get hls playlist : {}", v);
    // }

    // NamedFile 생성
    // let mut named_file = NamedFile::open(filepath)?;

    let cache_control = if filename.ends_with(".m3u8") {
        "max-age=1, public"
    } else if filename.ends_with(".mp4") || filename.ends_with(".m4s") {
        "max-age=3600, public"
    } else {
        "no-cache"
    };

    // HttpResponse로 변환 후 헤더 추가
    let response = NamedFile::open(filepath)?
        .use_last_modified(true)
        .prefer_utf8(true)
        .into_response(&req)
        .map_into_boxed_body()
        .customize()
        .insert_header((http::header::CACHE_CONTROL, cache_control));

    Ok(response.respond_to(&req).map_into_boxed_body())
    // let new_resp = NamedFile::open(filepath)?
    //     .customize()
    //     .respond_to(res.request())
    //     .map_into_boxed_body()
    //     .map_into_right_body();

    // Ok(response)
}
