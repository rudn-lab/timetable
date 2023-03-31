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
        .run(req, env)
        .await
}
