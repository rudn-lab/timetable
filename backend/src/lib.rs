use std::net::Ipv4Addr;

use actix_web::{get, web, App, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

pub async fn run(ip: Ipv4Addr, port: u16) -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind((ip, port))?
        .run()
        .await
}
