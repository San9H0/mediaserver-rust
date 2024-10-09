mod endpoints;
mod ingress;
mod hubs;

use std::sync::Arc;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};


#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    println!("Hello, world!");


    let hub = Arc::new(hubs::hub::Hub::new());
    endpoints::build(hub).await
}
