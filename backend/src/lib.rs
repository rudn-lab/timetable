use std::net::Ipv4Addr;

use actix_web::{get, post, web, App, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[post("/cache/faculties")]
async fn cache_facultis() -> impl Responder {
    todo!()
}

#[post("/cache/groups")]
async fn cache_groups() -> impl Responder {
    todo!()
}

#[post("/cache/timetables")]
async fn cache_timetables() -> impl Responder {
    todo!()
}

pub async fn run(ip: Ipv4Addr, port: u16) -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(index).service(hello))
        .bind((ip, port))?
        .run()
        .await
}
