use std::sync::{Arc, Mutex};

use delay_timer::prelude::*;

use crate::{database::Database, scraping};

/// Get all university faculties cron job, runs every 1 September
pub fn schedule_scrape_faculties(timer: &DelayTimer, db: Arc<Mutex<Database>>) {
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 9 *")
                .spawn_async_routine(move || {
                    let db = db.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 1 9 *\"");
                        log::info!("Scraping university faculties");
                        let faculties = scraping::scrape_faculties().await;
                        let mut db = db.lock().unwrap();
                        db.update_faculties(faculties);
                    }
                })
                .unwrap(),
        )
        .unwrap();
}

/// Get all studeng groups cron job, runs every 1st day of the month
pub fn schedule_scrape_groups(timer: &DelayTimer, db: Arc<Mutex<Database>>) {
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 * *")
                .spawn_async_routine(move || {
                    let _db = db.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 1 * *\"");
                        log::info!("Scraping student groups");
                        // scraping::scrape_groups().await;
                    }
                })
                .unwrap(),
        )
        .unwrap();
}

/// Get all studeng groups' current timetables cron job, runs every Monday
pub fn schedule_scrape_timetables(timer: &DelayTimer, db: Arc<Mutex<Database>>) {
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 * * 1")
                .spawn_async_routine(move || {
                    let _db = db.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 * * 1\"");
                        log::info!("Scraping current timetables");
                        // scraping::scrape_timetables().await
                    }
                })
                .unwrap(),
        )
        .unwrap();
}
