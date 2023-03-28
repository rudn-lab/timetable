use std::collections::HashMap;

use chrono::NaiveTime;
use worker::{kv::KvError, Env, Response, Result};
use worker::{Request, RouteContext};

use crate::asset::get_asset_data;
use crate::data::*;
use crate::templating::apply_template;
use crate::templating::context;
use crate::utils::auth;

pub async fn handle_index<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    if let Some(index_data) = get_asset_data(&ctx, "index.html").await {
        let mut tt: Timetable = Timetable::new();
        tt.insert(
            Day::Monday,
            vec![Event {
                name: String::from("Math"),
                offset: 0.0,
            }],
        );
        tt.insert(
            Day::Friday,
            vec![Event {
                name: String::from("CS"),
                offset: 3.0,
            }],
        );
        let ctx = context!(Timetable => tt);
        if let Ok(index) = String::from_utf8(index_data) {
            let index = apply_template("index.html", &index, ctx);
            return Response::from_html(index);
        }
    }
    todo!()
}

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
