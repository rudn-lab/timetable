use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;

use crate::{
    database::{
        models::{Faculty, Uuid},
        *,
    },
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
    }; // drop MutexGuard

    #[derive(Serialize)]
    struct PositiveResponse<'a> {
        faculties: Vec<Faculty>,
        links: HashMap<&'a str, &'a str>,
    }

    impl<'a> PositiveResponse<'a> {
        fn new(faculties: Vec<Faculty>) -> Self {
            Self {
                faculties,
                links: HashMap::from([("groups", "/{faculty_uuid}/groups")]),
            }
        }
    }

    if let Ok(faculties) = faculties {
        if !faculties.is_empty() {
            log::debug!("Returning faculties data from the database");
            return HttpResponse::Ok().json(PositiveResponse::new(faculties));
        }
    }
    // If the database is empty, scrape the data
    if let Some(faculties) = scraping::scrape_faculties().await {
        let mut db = db.lock().unwrap();
        if db.update_faculties(&faculties).is_ok() {
            log::debug!("Returning scraped faculties data");
            return HttpResponse::Ok().json(PositiveResponse::new(faculties));
        }
    }

    let msg = "Could not scrape faculties data from RUDN schedule webpage";
    log::warn!("{msg}");

    #[derive(Serialize)]
    struct NegativeResponse<'a> {
        reason: &'a str,
    }

    HttpResponse::NotFound().json(NegativeResponse { reason: msg })
}

/// This route returns all student groups for given faculty
#[get("/{faculty_uuid}/groups")]
pub async fn get_groups(
    faculty_uuid: web::Path<Uuid>,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    let groups = {
        let mut db = db.lock().unwrap();
        db.get_groups_for_faculty(&faculty_uuid)
    };

    if let Ok(groups) = groups {
        if !groups.is_empty() {
            log::debug!("Returning groups data from the database");
            return HttpResponse::Ok().json(groups);
        }
    }
    // We do not have group data in DB, scrape anew
    if let Some(scraped_groups) = scraping::scrape_group(&faculty_uuid).await {
        let mut db = db.lock().unwrap();
        if db.update_groups(&scraped_groups).is_ok() {
            log::debug!("Returning scraped groups data");
            return HttpResponse::Ok().json(scraped_groups);
        }
    }

    let msg = format!("Could not scrape group data for this faculty: {faculty_uuid}");
    log::warn!("{msg}");

    #[derive(Serialize)]
    struct NegativeResponse<'a> {
        reason: &'a str,
        links: HashMap<&'a str, &'a str>,
    }

    HttpResponse::NotFound().json(NegativeResponse {
        reason: &msg,
        links: HashMap::from([("faculties", "/faculties")]),
    })
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
        db.get_timetable_for_group(&group_uuid)
    };

    if let Ok(timetable) = timetable {
        if !timetable.is_empty() {
            log::debug!("Returning timetable data from the database");
            return HttpResponse::Ok().json(timetable);
        }
    }

    if let Some(scraped_timetable) = scraping::scrape_timetable(&group_uuid).await {
        let mut db = db.lock().unwrap();
        if db.update_timetable(&scraped_timetable).is_ok() {
            log::debug!("Returning scraped timetable data");
            return HttpResponse::Ok().json(scraped_timetable);
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
