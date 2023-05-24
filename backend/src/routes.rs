use std::sync::{Arc, Mutex};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::{
    database::{models::Uuid, *},
    scraping,
};

// TODO: URGENT! Rewrite so the server doesn't trigger updates on itself
// instead, suggest the user do them

// Todo: make use of hal+json to give links to resources

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
                    log::warn!("Could not find any faculties data");
                    HttpResponse::NonAuthoritativeInformation()
                        .insert_header((
                            "Warning",
                            "110 timetable-backend \"Could not ask RUDN web page\"",
                        ))
                        .finish()
                }
            }
        }
    }
}

/// This route returns all student groups for given faculty
#[get("/groups/{faculty_uuid}")]
pub async fn get_groups(
    faculty_uuid: web::Path<Uuid>,
    req: HttpRequest,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    // Get groups from DB
    let groups = {
        let mut db = db.lock().unwrap();
        db.get_groups_by_faculty(&faculty_uuid)
    };
    if !groups.is_empty() {
        log::debug!("Returning groups data from the database");
        HttpResponse::Ok().body(serde_json::to_string(&groups).unwrap_or_default())
    } else {
        // We do not have group data in DB, scrape anew
        match scraping::scrape_group(&faculty_uuid).await {
            Some(scraped_groups) => {
                // Make into a closure, to later reuse
                let update_groups_db = || {
                    let mut db = db.lock().unwrap();
                    db.update_groups(&scraped_groups).ok()
                };
                match update_groups_db() {
                    Some(_) => {
                        log::debug!("Returning scraped groups data");
                        HttpResponse::Ok()
                            .body(serde_json::to_string(&scraped_groups).unwrap_or_default())
                    }
                    None => {
                        // Something is wrong with faculties table, update it.
                        log::debug!("Triggering faculties update");
                        let _ = reqwest::get(format!(
                            "{}:{}/faculties",
                            req.connection_info().scheme(),
                            req.headers().get("host").unwrap().to_str().unwrap()
                        ))
                        .await;

                        // Update DB
                        let _ = update_groups_db();
                        log::debug!("Returning scraped groups data, after faculties update");
                        HttpResponse::Ok()
                            .body(serde_json::to_string(&scraped_groups).unwrap_or_default())
                    }
                }
            }
            None => {
                log::warn!("Could not find any groups data");
                return HttpResponse::NonAuthoritativeInformation()
                    .insert_header((
                        "Warning",
                        "110 timetable-backend \"Could not ask RUDN web page\"",
                    ))
                    .finish();
            }
        }
    }
}

/// This route returns current week timetable for specified group
/// Accepts a query string with `group` as parameter;
#[get("/timetable/{group_uuid}")]
pub async fn get_timetable(
    group_uuid: web::Path<Uuid>,
    req: HttpRequest,
    db: web::Data<Arc<Mutex<Database>>>,
) -> impl Responder {
    let timetable = {
        let mut db = db.lock().unwrap();
        db.get_timetable(&group_uuid)
    };

    if !timetable.is_empty() {
        log::debug!("Returning timetable data from the database");
        HttpResponse::Ok().body(serde_json::to_string(&timetable).unwrap_or_default())
    } else {
        // Scraping new
        match scraping::scrape_timetable(&group_uuid).await {
            Some(scraped_timetable) => {
                let update_timetable_db = || {
                    let mut db = db.lock().unwrap();
                    db.update_timetable(&scraped_timetable).ok()
                };

                match update_timetable_db() {
                    Some(_) => {
                        log::debug!("Returning scraped timetable data");
                        HttpResponse::Ok()
                            .body(serde_json::to_string(&scraped_timetable).unwrap_or_default())
                    }
                    None => {
                        let self_root_url = format!(
                            "{}:{}",
                            req.connection_info().scheme(),
                            req.headers().get("host").unwrap().to_str().unwrap()
                        );
                        // Something is wrong with groups table, update it.
                        log::debug!("Triggering groups update");
                        let get_faculty_for_group = || {
                            let mut db = db.lock().unwrap();
                            db.get_faculty_for_group(&group_uuid)
                        };
                        if let Some(faculty_uuid) = get_faculty_for_group() {
                            let _ = reqwest::get(format!("{self_root_url}/groups/{faculty_uuid}"))
                                .await;

                            // Update DB
                            let _ = update_timetable_db();
                            log::debug!("Returning scraped groups data, after groups update");
                            HttpResponse::Ok()
                                .body(serde_json::to_string(&scraped_timetable).unwrap_or_default())
                        } else {
                            log::debug!("Triggering faculties update");
                            let _ = reqwest::get(format!("{self_root_url}/faculties")).await;

                            // WARN: yes I know it's literally a copy, it _is_ bad
                            // TODO: refactor
                            if let Some(faculty_uuid) = get_faculty_for_group() {
                                let _ =
                                    reqwest::get(format!("{self_root_url}/groups/{faculty_uuid}"))
                                        .await;

                                // Update DB
                                let _ = update_timetable_db();
                                log::debug!(
                                    "Returning scraped groups data, after faculties and groups update"
                                );
                                HttpResponse::Ok().body(
                                    serde_json::to_string(&scraped_timetable).unwrap_or_default(),
                                )
                            } else {
                                HttpResponse::BadRequest()
                                    .body("This group does not belong to any faculty")
                            }
                        }
                    }
                }
            }
            None => {
                log::warn!("Could not find any groups data");
                return HttpResponse::NonAuthoritativeInformation()
                    .insert_header((
                        "Warning",
                        "110 timetable-backend \"Could not ask RUDN web page\"",
                    ))
                    .finish();
            }
        }
    }
}
