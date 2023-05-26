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

    if !faculties.is_empty() {
        log::debug!("Returning faculties data from the database");
        HttpResponse::Ok().json(faculties)
    } else {
        // If the database is empty, scrape the data
        match scraping::scrape_faculties().await {
            Some(faculties) => {
                log::debug!("Scraping new faculties data");
                let mut db = db.lock().unwrap();
                db.update_faculties(&faculties);
                HttpResponse::Ok().json(faculties)
            }
            None => {
                let msg = "Could not scrape faculties data from RUDN schedule webpage";
                log::warn!("{msg}");

                #[derive(Serialize)]
                struct Response<'a> {
                    reason: &'a str,
                }

                let body = Response { reason: msg };

                HttpResponse::NotFound().json(body)
            }
        }
    }
}

/// This route returns all student groups for given faculty
#[get("/{faculty_uuid}/groups")]
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
        return HttpResponse::Ok().json(groups);
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
                return HttpResponse::Ok().json(scraped_groups);
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

    HttpResponse::NotFound().json(body)
}

/// This route returns current week timetable for specified group
/// Accepts a query string with `group` as parameter;
#[get("/{group_uuid}/timetable")]
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
        return HttpResponse::Ok().json(timetable);
    } else {
        // Scraping new
        if let Some(scraped_timetable) = scraping::scrape_timetable(&group_uuid).await {
            let update_timetable_db = || {
                let mut db = db.lock().unwrap();
                db.update_timetable(&scraped_timetable).ok()
            };

            if update_timetable_db().is_some() {
                log::debug!("Returning scraped timetable data");
                return HttpResponse::Ok().json(scraped_timetable);
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
            ("groups", "/{faculty_uuid}/groups"),
        ]),
    };

    HttpResponse::NotFound().json(body)
}
