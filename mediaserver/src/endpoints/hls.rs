use crate::{egress::servers::hls::HlsPath, endpoints::Container};
use actix_files::NamedFile;
use actix_web::{web, HttpResponse, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};

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
    handler: web::Data<Container>,
    path: web::Path<(String, String)>,
    query: web::Query<HlsQuery>,
) -> actix_web::Result<NamedFile> {
    let (session_id, path) = path.into_inner();
    let query = query.into_inner();

    let _session = match handler.hls_server.get_session(&session_id).await {
        Ok(session) => session,
        Err(err) => {
            log::error!("get hls error:{}", err);
            return Err(actix_web::error::ErrorNotFound("Session not found"));
        }
    };

    let hls_path = HlsPath::new(session_id.to_string());
    let Ok(filepath2) = hls_path.get_path(&path) else {
        println!("invalid path...");
        return Err(actix_web::error::ErrorNotFound("File not found"));
    };
    let is_master = path == "index.m3u8";
    let is_playlist = path == "video.m3u8";
    let is_video = path.ends_with(".mp4") || path.ends_with(".m4s");
    if !is_video && !is_master && !is_playlist {
        println!("Bad request");
        return Err(actix_web::error::ErrorBadRequest("Bad request"));
    }

    if let (Some(msn), Some(part)) = (query._hls_msn, query._hls_part) {
        // 두 값이 모두 있는 경우 처리
        let mut rx = _session.service.subscribe_signal();
        if rx.changed().await.is_ok() {
            let (recv_msn, recv_part) = *rx.borrow();
        }
    }

    let filepath = if is_master {
        hls_path.get_master_path()
    } else if is_playlist {
        hls_path.get_playlist_path()
    } else {
        // is_video
        hls_path.get_video_path(&path)
    };

    println!("filepath:{}, filepath2:{}", filepath, filepath2);

    // if is_playlist {
    //     let v = tokio::fs::read_to_string(filepath.to_string()).await?;
    //     log::info!("get hls playlist : {}", v);
    // }

    Ok(NamedFile::open(filepath)?)
}
