use worker::*;

mod asset;
mod data;
mod handlers;
mod templates;
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
        .get_async("/", handlers::handle_root)
        // .get_async("/", |_, context| asset::serve(context))
        .post_async("/update", handlers::handle_update)
        .get("/worker-version", handlers::handle_version)
        .run(req, env)
        .await
}
