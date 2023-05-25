use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

use crate::{
    database::{models::Uuid, *},
    scraping,
};

/// This route returns all faculties of the RUDN University from the database,
/// if there is no faculties stored it scrapes info from the web and returns that.
/// However, if at least one faculty is left in the database this function will not scrape the rest
#[get("/faculties")]
pub async fn get_faculties(db: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let faculties = {
        let mut db = db.lock().unwrap();
        db.get_faculties()
    };

    match faculties {
        Some(faculties) => {
            log::debug!("Returning faculties data from the database");
            HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
        }
        None => {
            // If the database is empty, scrape the data
            match scraping::scrape_faculties().await {
                Some(faculties) => {
                    log::debug!("Scraping new faculties data");
                    let mut db = db.lock().unwrap();
                    db.update_faculties(&faculties);
                    HttpResponse::Ok().body(serde_json::to_string(&faculties).unwrap_or_default())
                }
                None => {
                    let msg = "Could not scrape faculties data from RUDN schedule web-page";
                    log::warn!("{msg}");

                    #[derive(Serialize)]
                    struct Response<'a> {
                        reason: &'a str,
                    }

                    let body = Response { reason: msg };

                    HttpResponse::NotFound().json(serde_json::to_string(&body).unwrap())
                }
            }
        }
    }
}

/// This route returns all student groups for given faculty
#[get("/groups/{faculty_uuid}")]
pub async fn get_groups(
    faculty_uuid: web::Path<Uuid>,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    // Get groups from DB
    let groups = {
        let mut db = db.lock().unwrap();
        db.get_groups_by_faculty(&faculty_uuid)
    };
    if !groups.is_empty() {
        log::debug!("Returning groups data from the database");
        return HttpResponse::Ok().body(serde_json::to_string(&groups).unwrap_or_default());
    } else {
        // We do not have group data in DB, scrape anew
        if let Some(scraped_groups) = scraping::scrape_group(&faculty_uuid).await {
            // Make into a closure, to later reuse
            let update_groups_db = || {
                let mut db = db.lock().unwrap();
                db.update_groups(&scraped_groups).ok()
            };
            if update_groups_db().is_some() {
                log::debug!("Returning scraped groups data");
                return HttpResponse::Ok()
                    .body(serde_json::to_string(&scraped_groups).unwrap_or_default());
            }
        }
    }

    let msg = format!("Could not scrape group data for this faculty: {faculty_uuid}");
    log::warn!("{msg}");

    #[derive(Serialize)]
    struct Response<'a> {
        reason: &'a str,
        links: HashMap<&'static str, &'static str>,
    }

    let body = Response {
        reason: &msg,
        links: HashMap::from([("faculties", "/faculties")]),
    };

    HttpResponse::NotFound().json(serde_json::to_string(&body).unwrap())
}

/// This route returns current week timetable for specified group
/// Accepts a query string with `group` as parameter;
#[get("/timetable/{group_uuid}")]
pub async fn get_timetable(
    group_uuid: web::Path<Uuid>,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    let timetable = {
        let mut db = db.lock().unwrap();
        db.get_timetable(&group_uuid)
    };

    if !timetable.is_empty() {
        log::debug!("Returning timetable data from the database");
        return HttpResponse::Ok().body(serde_json::to_string(&timetable).unwrap_or_default());
    } else {
        // Scraping new
        if let Some(scraped_timetable) = scraping::scrape_timetable(&group_uuid).await {
            let update_timetable_db = || {
                let mut db = db.lock().unwrap();
                db.update_timetable(&scraped_timetable).ok()
            };

            if update_timetable_db().is_some() {
                log::debug!("Returning scraped timetable data");
                return HttpResponse::Ok()
                    .body(serde_json::to_string(&scraped_timetable).unwrap_or_default());
            }
        }
    }

    let msg = format!("Could not scrape timetable data for this group: {group_uuid}");
    log::warn!("{msg}");

    #[derive(Serialize)]
    struct Response<'a> {
        reason: &'a str,
        links: HashMap<&'static str, &'static str>,
    }

    let body = Response {
        reason: &msg,
        links: HashMap::from([
            ("faculties", "/faculties"),
            ("groups", "/groups/{faculty_uuid}"),
        ]),
    };

    HttpResponse::NotFound().json(serde_json::to_string(&body).unwrap())
}
