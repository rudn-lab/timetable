use actix_web::{middleware::Logger, services, App, HttpServer};
use delay_timer::prelude::*;
use std::net::Ipv4Addr;

mod routes;
mod scheduling;
mod scraping;

pub async fn run_server(ip: Ipv4Addr, port: u16) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
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

pub async fn run_scheduler() {
    let timer = DelayTimerBuilder::default().build();
    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 * * * * *")
                .spawn_async_routine(|| async {
                    log::info!("Running test cron job \"0 * * * * *\"");
                    print!("Hello, World!");
                })
                .unwrap(),
        )
        .unwrap();

    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 * * 1")
                .spawn_async_routine(|| async {
                    log::info!("Running cron job \"0 0 0 * * 1\"");
                    scraping::scrape_timetables().await
                })
                .unwrap(),
        )
        .unwrap();

    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 * *")
                .spawn_async_routine(|| async {
                    log::info!("Running cron job \"0 0 0 1 * *\"");
                    scraping::scrape_groups().await;
                })
                .unwrap(),
        )
        .unwrap();

    let _ = timer
        .insert_task(
            TaskBuilder::default()
                .set_frequency_repeated_by_cron_str("0 0 0 1 9 *")
                .spawn_async_routine(|| async {
                    log::info!("Running cron job \"0 0 0 1 9 *\"");
                    scraping::scrape_faculties().await;
                })
                .unwrap(),
        )
        .unwrap();
}
