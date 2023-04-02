use actix_web::{middleware::Logger, services, web, App, HttpServer};
use database::Database;
use delay_timer::prelude::DelayTimerBuilder;
use std::{
    net::Ipv4Addr,
    sync::{Arc, Mutex},
};

mod database;
mod routes;
mod scheduling;
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

    scheduling::schedule_scrape_faculties(&timer, db.clone());
    scheduling::schedule_scrape_groups(&timer, db.clone());
    scheduling::schedule_scrape_timetables(&timer, db);
}
