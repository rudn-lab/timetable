use std::collections::HashMap;

use chrono::NaiveTime;
use data::Day;
use worker::*;

mod asset;
mod data;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    let router = Router::new();
    router
        .get_async("/", |_, ctx| asset::serve_asset(ctx))
        .get_async("/:asset", |_, ctx| asset::serve_asset(ctx))
        .post_async("/update", handle_update)
        .get("/worker-version", |_, ctx| {
            Response::ok(ctx.var("WORKERS_RS_VERSION")?.to_string())
        })
        .run(req, env)
        .await
}

pub async fn handle_update<D>(mut req: Request, ctx: RouteContext<D>) -> Result<Response> {
    let kv = ctx.kv("TIMETABLE_KV")?;
    let data: HashMap<Day, [NaiveTime; 2]> = req.json().await?;
    for (day, time) in data {
        kv.put(&serde_json::to_string(&day)?, serde_json::to_string(&time)?)?
            .execute()
            .await?;
    }

    Response::ok("Received new timetable")
}
