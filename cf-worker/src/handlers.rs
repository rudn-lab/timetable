use std::collections::HashMap;

use chrono::NaiveTime;
use worker::kv::KvStore;
use worker::{kv::KvError, Response, Result};
use worker::{Request, RouteContext};

use crate::asset::get_asset_data;
use crate::data::*;
use crate::templating::apply_template;
use crate::templating::context;
use crate::utils::auth;

pub async fn handle_index<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    if let Some(index_data) = get_asset_data(&ctx, "index.html").await {
        if let Ok(kv) = ctx.kv("TIMETABLE_KV") {
            let tt = load_timetable(&kv).await;
            let ctx = context!(Timetable => tt);
            if let Ok(index) = String::from_utf8(index_data) {
                let index = apply_template("index.html", &index, ctx);
                return Response::from_html(index);
            }
        }
    }
    todo!()
}

async fn load_timetable(kv: &KvStore) -> Timetable {
    let mut tt = Timetable::new();
    for day in Day::values() {
        let possible_lab_time: std::result::Result<Option<[NaiveTime; 2]>, KvError> =
            kv.get(&serde_json::to_string(&day).unwrap()).json().await;
        if let Ok(Some(lab_time)) = possible_lab_time {
            let calc_time_offset = |time: NaiveTime| -> TimeOffset {
                ((time - *FIRST_CLASS_START).num_minutes() as f64)
                    / (CLASS_DURATION.num_minutes() as f64)
            };
            let class_start_offset = calc_time_offset(lab_time[0]);
            let class_duration = calc_time_offset(lab_time[1]) - class_start_offset;

            let event = Event {
                name: String::from("Lab"),
                start_offset: class_start_offset,
                duration: class_duration,
                start_time: lab_time[0],
                end_time: lab_time[1],
            };

            tt.insert(day, vec![event]);
        }
    }
    tt
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
