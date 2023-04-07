use std::sync::{Arc, Mutex};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::{
    database::{models::Uuid, *},
    scraping,
};

// Todo: make use of hal+json to give links to resources

/// This route returns all faculties of the RUDN University from the database,
/// if there is no faculties stored it scrapes info from the web and returns that.
/// However, if some but not all faculties were deleted from the database
/// this method will _not_ scrape them back
#[get("/faculties")]
pub async fn get_faculties(db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let query_res = {
        let mut db = db.lock().unwrap();
        db.get_faculties(FacultiesSelection::Total)
    };

    match query_res {
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

#[derive(Debug, Deserialize)]
struct Params {
    faculties: Vec<Uuid>,
}

#[get("/groups")]
pub async fn get_groups(req: HttpRequest, db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    match serde_qs::from_str::<Params>(req.query_string()) {
        Ok(params) => {
            let query_res = {
                let mut db = db.lock().unwrap();
                db.get_faculties(FacultiesSelection::Partial(&params.faculties))
            };

            if let Some(faculties) = query_res {
                let result = scraping::scrape_groups(faculties).await;
                HttpResponse::Ok().body(serde_json::to_string(&result).unwrap_or_default())
                // Todo: update the database
            } else {
                // Update itself
                let _ = reqwest::get(format!(
                    "{}:{}/faculties",
                    req.connection_info().scheme(),
                    req.headers().get("host").unwrap().to_str().unwrap()
                ))
                .await;
                let query_res = {
                    let mut db = db.lock().unwrap();
                    db.get_faculties(FacultiesSelection::Partial(&params.faculties))
                        .unwrap_or_default()
                };
                let result = scraping::scrape_groups(query_res).await;
                HttpResponse::Ok().body(serde_json::to_string(&result).unwrap_or_default())
            }
        }
        Err(e) => {
            log::error!("{e:?}");
            HttpResponse::BadRequest().body("Invalid url query")
        }
    }
}

#[get("/timetables")]
pub async fn get_timetables(_db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    scraping::scrape_timetables().await;
    HttpResponse::Accepted().finish()
}
