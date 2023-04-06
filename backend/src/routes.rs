use std::sync::{Arc, Mutex};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::{database::Database, scraping};

mod util;
use util::*;

/// This route returns all faculties of the RUDN University from the database,
/// if there is no faculties stored it scrapes info from the web and returns that.
/// However, if some but not all faculties were deleted from the database
/// this method will _not_ scrape them back
#[get("/faculties")]
pub async fn get_faculties(db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let res = {
        let mut db = db.lock().unwrap();
        db.get_faculties()
    };

    match res {
        Some(faculties) => {
            HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
        }
        None => {
            let faculties = scraping::scrape_faculties().await;
            let mut db = db.lock().unwrap();
            db.update_faculties(&faculties);
            HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
        }
    }
}

#[get("/groups")]
pub async fn get_groups(req: HttpRequest, _db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    if let Some(query) = req.uri().query() {
        let params = parse_url_params(query);
        println!("{params:?}");
    }
    scraping::scrape_groups().await;
    HttpResponse::Accepted().finish()
}

#[get("/timetables")]
pub async fn get_timetables(_db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    scraping::scrape_timetables().await;
    HttpResponse::Accepted().finish()
}
