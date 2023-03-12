use chrono::NaiveTime;
use std::collections::HashMap;

use serde_json;
use worker::*;

mod data;
use data::*;

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
        .get_async("/", handle_root)
        .post_async("/update", handle_update)
        .get("/worker-version", handle_version)
        .run(req, env)
        .await
}

pub async fn handle_root<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    if let Ok(kv) = ctx.kv("TIMETABLE_KV") {
        if let Ok(Some(msg)) = kv.get("message").text().await {
            return Response::ok(msg);
        }
    }

    Response::ok("Hello from Workers!")
}

pub async fn handle_update<D>(mut req: Request, ctx: RouteContext<D>) -> Result<Response> {
    let kv = ctx.kv("TIMETABLE_KV")?;
    let data: HashMap<Day, [NaiveTime; 2]> = req.json().await?;
    for (day, time) in data {
        kv.put(&serde_json::to_string(&day)?, serde_json::to_string(&time)?)?
            .execute()
            .await?;
    }

    return Response::ok("Received new timetable");
}

pub fn handle_version<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
    Response::ok(version)
}
