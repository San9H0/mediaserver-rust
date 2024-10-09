use actix_web::{post, web, HttpResponse, Responder};

#[derive(Clone)]
pub struct WhepHandler {
    pub args: String,
}

impl WhepHandler {
    pub fn new(args: String) -> Self {
        WhepHandler { args }
    }
}


// #[post("/v1/whep")]
pub async fn handle_whep(handler : web::Data<WhepHandler>, offer: String) -> impl Responder {
    println!("handler:{}", handler.args);
    println!("{}", offer);
    HttpResponse::Ok().body("success")
}