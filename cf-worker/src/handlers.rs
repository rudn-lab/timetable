use std::collections::HashMap;

use chrono::NaiveTime;
use worker::{kv::KvError, Env, Response, Result};
use worker::{Request, RouteContext};

use crate::data::Day;
use crate::utils::auth;

pub async fn handle_update<D>(mut req: Request, ctx: RouteContext<D>) -> Result<Response> {
    if let Some(err_resp) = auth(&req, &ctx).await {
        return err_resp;
    }

    let kv = ctx.kv("TIMETABLE_KV")?;
    for day in Day::values() {
        kv.delete(&serde_json::to_string(&day)?).await?;
    }
    let data: HashMap<Day, [NaiveTime; 2]> = req.json().await?;
    for (day, time) in data {
        kv.put(&serde_json::to_string(&day)?, serde_json::to_string(&time)?)?
            .execute()
            .await?;
    }

    Response::ok("Received new timetable")
}

pub async fn cache_student_goups(env: &Env) -> Result<String> {
    let kv = env.kv("RUDN_FACULTIES")?;
    let list = kv.list().execute().await?.keys;
    for key in list {
        let _uuid = kv
            .get(&key.name)
            .text()
            .await?
            .ok_or(KvError::InvalidKvStore(format!(
                "No such key error: key={key:?}"
            )));
    }

    todo!("get uuid; request all groups for this faculty; add them to new kv")
}

pub async fn cache_student_timetables() -> Result<String> {
    todo!("for each faculty and each group request timetable; add to kv")
}
