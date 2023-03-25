use std::collections::HashMap;

use chrono::NaiveTime;
use data::Day;
use worker::{kv::KvError, *};

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
        .post_async("/cache-groups", |_, ctx| async move {
            match cache_student_goups(&ctx.env).await {
                Ok(stat) => Response::ok(stat),
                Err(msg) => Response::error(msg.to_string(), 500),
            }
        })
        .post_async("/cache-timetables", |_, _| async move {
            let _res = cache_student_timetables().await;
            Response::ok("cached")
        })
        .run(req, env)
        .await
}

async fn handle_update<D>(mut req: Request, ctx: RouteContext<D>) -> Result<Response> {
    let kv = ctx.kv("TIMETABLE_KV")?;
    let data: HashMap<Day, [NaiveTime; 2]> = req.json().await?;
    for (day, time) in data {
        kv.put(&serde_json::to_string(&day)?, serde_json::to_string(&time)?)?
            .execute()
            .await?;
    }

    Response::ok("Received new timetable")
}

pub async fn cron(event: ScheduledEvent, env: Env, _: ScheduleContext) {
    let cron = event.cron();
    let res = match cron.as_str() {
        "0 0 1 * *" => cache_student_goups(&env).await,
        "0 0 * * mon" => cache_student_timetables().await,
        _ => unreachable!("All cron jobs should be covered"),
    };

    match res {
        Ok(stat) => console_log!("Cron event '{cron}' log: {stat}"),
        Err(msg) => console_error!("Cron event '{cron}' error: {msg}"),
    }
}

async fn cache_student_goups(env: &Env) -> Result<String> {
    let kv = env.kv("RUDN_FACULTIES")?;
    let list = kv.list().execute().await?.keys;
    for key in list {
        let uuid = kv
            .get(&key.name)
            .text()
            .await?
            .ok_or(KvError::InvalidKvStore(format!(
                "No such key error: key={key:?}"
            )));
    }

    todo!("get uuid; request all groups for this faculty; add them to new kv")
}

async fn cache_student_timetables() -> Result<String> {
    todo!("for each faculty and each group request timetable; add to kv")
}
