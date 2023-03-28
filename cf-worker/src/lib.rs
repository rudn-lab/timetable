use worker::*;

mod asset;
mod data;
mod handlers;
mod templating;
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
        .get_async("/", handlers::handle_index)
        .get_async("/:asset", |_, ctx| asset::serve_asset(ctx))
        .post_async("/update", handlers::handle_update)
        .get("/worker-version", |_, ctx| {
            Response::ok(ctx.var("WORKERS_RS_VERSION")?.to_string())
        })
        .post_async("/cache-groups", |_, ctx| async move {
            match handlers::cache_student_goups(&ctx.env).await {
                Ok(stat) => Response::ok(stat),
                Err(msg) => Response::error(msg.to_string(), 500),
            }
        })
        .post_async("/cache-timetables", |_, _| async move {
            let _res = handlers::cache_student_timetables().await;
            Response::ok("cached")
        })
        .run(req, env)
        .await
}

#[event(scheduled)]
pub async fn cron(event: ScheduledEvent, env: Env, _: ScheduleContext) {
    let cron = event.cron();
    let res = match cron.as_str() {
        "0 0 1 * *" => handlers::cache_student_goups(&env).await,
        "0 0 * * mon" => handlers::cache_student_timetables().await,
        _ => unreachable!("All cron jobs should be covered"),
    };

    match res {
        Ok(stat) => console_log!("Cron event '{cron}' log: {stat}"),
        Err(msg) => console_error!("Cron event '{cron}' error: {msg}"),
    }
}
