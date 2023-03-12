use crate::data::*;
use chrono::NaiveTime;
use serde_json;
use std::collections::HashMap;
use worker::*;

pub async fn handle_root<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    if let Ok(kv) = ctx.kv("__STATIC_CONTENT") {
        let keys = kv.list().execute().await?.keys;
        console_log!("{:?}", keys);
        if let Ok(Some(msg)) = kv.get(&keys[0].name).text().await {
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
