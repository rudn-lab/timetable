use std::sync::{Arc, Mutex};

use actix_web::{post, web, HttpResponse, Responder};

use crate::{database::Database, scraping};

#[post("/cache/faculties")]
async fn cache_facultis(db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let faculties = scraping::scrape_faculties().await;

    let mut db = db.lock().unwrap();
    db.update_faculties(faculties);

    HttpResponse::Accepted().finish()
}

#[post("/cache/groups")]
async fn cache_groups() -> impl Responder {
    scraping::scrape_groups().await;
    HttpResponse::Accepted().finish()
}

#[post("/cache/timetables")]
async fn cache_timetables() -> impl Responder {
    scraping::scrape_timetables().await;
    HttpResponse::Accepted().finish()
}
