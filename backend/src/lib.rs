use actix_web::{middleware::Logger, services, web, App, HttpServer};
use database::Database;
use delay_timer::prelude::*;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

mod database;
mod routes;
mod scraping;

pub async fn init(ip: Ipv4Addr, port: u16) -> std::io::Result<()> {
    let db_conn = Database::establish_connection();
    let db_conn = Arc::new(Mutex::new(db_conn));
    let db_conn_web = web::Data::new(Arc::clone(&db_conn));

    run_scheduler(Arc::clone(&db_conn)).await;

    run_server(ip, port, db_conn_web).await
}

async fn run_server(
    ip: Ipv4Addr,
    port: u16,
    db_conn: web::Data<Arc<Mutex<Database>>>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(db_conn.clone())
            // https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html#format
            .wrap(Logger::default())
            .service(services![
                routes::cache_facultis,
                routes::cache_facultis,
                routes::cache_timetables
            ])
    })
    .bind((ip, port))?
    .run()
    .await
}

async fn run_scheduler(db: Arc<Mutex<Database>>) {
    let timer = DelayTimerBuilder::default().build();

    let db_clone = db.clone();
    // Test cron job, runs every minute
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 * * * * *")
                .spawn_async_routine(move || {
                    let _db = db_clone.clone();
                    async move {
                        log::info!("Running test cron job \"0 * * * * *\"");
                    }
                })
                .unwrap(),
        )
        .unwrap();

    let db_clone = db.clone();
    // Get faculties cron job, runs every 1st of September
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 9 *")
                .spawn_async_routine(move || {
                    let db = db_clone.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 1 9 *\"");
                        let faculties = scraping::scrape_faculties().await;
                        let mut db = db.lock().unwrap();
                        db.update_faculties(faculties);
                    }
                })
                .unwrap(),
        )
        .unwrap();

    let db_clone = db.clone();
    // Get studeng groups cron job, runs every 1st day of any month
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 * *")
                .spawn_async_routine(move || {
                    let _db = db_clone.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 1 * *\"");
                        scraping::scrape_groups().await;
                    }
                })
                .unwrap(),
        )
        .unwrap();

    // Get studeng groups' timetables cron job, runs every Monday
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 * * 1")
                .spawn_async_routine(move || {
                    let _db = db.clone();
                    async move {
                        log::info!("Running cron job \"0 0 0 * * 1\"");
                        scraping::scrape_timetables().await
                    }
                })
                .unwrap(),
        )
        .unwrap();
}
