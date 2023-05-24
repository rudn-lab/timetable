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
///
/// # Implementation
/// 1. Return faculties if we have them in the DB
/// 2. If we have none, ask the timetable website and update DB;
/// 3. Return data
#[get("/faculties")]
pub async fn get_faculties(db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let faculties = {
        let mut db = db.lock().unwrap();
        db.get_faculties()
    };

    if !faculties.is_empty() {
        HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
    } else {
        // If the database is empty, scrape the data
        match scraping::scrape_faculties().await {
            Some(faculties) => {
                let mut db = db.lock().unwrap();
                db.update_faculties(&faculties);
                HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
            }
            None => HttpResponse::NonAuthoritativeInformation()
                .insert_header((
                    "Warning",
                    "110 timetable-backend \"Could not ask RUDN web page\"",
                ))
                .finish(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GroupsParams {
    faculties: Vec<Uuid>,
}

/// This route returns all student groups for given faculties
/// Accepts a query string with `faculties[]` as parameter;
/// several may be supplied `/groups?faculties?[]=<faculty-uuid-1>&faculties[]=<faculty-uuid-2>`
///
/// # Implementation
/// 1. If we have the data already, just return them
/// 2. If we have the faculties, scrape group data from the timetable website; update DB and
///    return new data
/// 3. If we do not have faculties, scrape them; scrape groups; update DB and return data
#[get("/groups")]
pub async fn get_groups(req: HttpRequest, db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    match serde_qs::from_str::<GroupsParams>(req.query_string()) {
        Ok(params) => {
            // Get groups from DB
            let groups = {
                let mut db = db.lock().unwrap();
                db.get_groups_by_faculty(&params.faculties)
            };
            if !groups.is_empty() {
                HttpResponse::Ok().body(serde_json::to_string(&groups).unwrap_or_default())
            } else {
                // We do not have group data in DB, scrape anew
                let scraped_groups = scraping::scrape_groups(&params.faculties).await;

                // Make into a closure, to later reuse
                let update_groups = || {
                    let mut db = db.lock().unwrap();
                    db.update_groups(
                        &scraped_groups
                            .values()
                            .flat_map(|el| el.clone())
                            .collect::<Vec<_>>(),
                    )
                };
                let groups_res = update_groups();
                match groups_res {
                    Ok(_) => HttpResponse::Ok()
                        .body(serde_json::to_string(&scraped_groups).unwrap_or_default()),
                    Err(_) => {
                        // Something is wrong with faculties table
                        // Update it.
                        // Trigger faculties db update on itself
                        let _ = reqwest::get(format!(
                            "{}:{}/faculties",
                            req.connection_info().scheme(),
                            req.headers().get("host").unwrap().to_str().unwrap()
                        ))
                        .await;

                        // Update DB
                        let _ = update_groups();
                        HttpResponse::Ok()
                            .body(serde_json::to_string(&scraped_groups).unwrap_or_default())
                    }
                }
            }
        }
        Err(e) => {
            log::error!("{e:?}");
            HttpResponse::BadRequest().body("Invalid url query")
        }
    }
}

#[derive(Debug, Deserialize)]
struct TimetableParams {
    group: Uuid,
}

/// This route returns current week timetable for specified group
/// Accepts a query string with `group` as parameter;
///
/// # Implementation
/// 1. If we have the data already, just return it
/// 2. If it is missing, scrape, update db and return
/// 3. If there's an error with the db (faculty or group is not there) scrape them by triggering
///    `/groups` enpoint, update the db and return data
#[get("/timetable")]
pub async fn get_timetable(
    req: HttpRequest,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    match serde_qs::from_str::<TimetableParams>(req.query_string()) {
        Ok(params) => {
            let timetable = {
                let mut db = db.lock().unwrap();
                db.get_timetable(&params.group)
            };

            if !timetable.is_empty() {
                HttpResponse::Ok().body(serde_json::to_string(&timetable).unwrap_or_default())
            } else {
                // Scraping new
                match scraping::scrape_timetable(params.group).await {
                    Ok(scraped_timetable) => {
                        let mut db = db.lock().unwrap();
                        match db.update_timetable(&scraped_timetable) {
                            Ok(_) => HttpResponse::Ok().body(
                                serde_json::to_string(&scraped_timetable).unwrap_or_default(),
                            ),
                            Err(_) => HttpResponse::NotFound().finish(),
                        }
                    }
                    Err(_) => HttpResponse::NotFound().finish(),
                }
            }
        }
        Err(e) => {
            log::error!("{e:?}");
            HttpResponse::BadRequest().body("Invalid url query")
        }
    }
}
