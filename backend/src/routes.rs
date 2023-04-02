use actix_web::{post, HttpResponse, Responder};

#[post("/cache/faculties")]
async fn cache_facultis() -> impl Responder {
    HttpResponse::Accepted().finish()
}

#[post("/cache/groups")]
async fn cache_groups() -> impl Responder {
    HttpResponse::Accepted().finish()
}

#[post("/cache/timetables")]
async fn cache_timetables() -> impl Responder {
    HttpResponse::Accepted().finish()
}
