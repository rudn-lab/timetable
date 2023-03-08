use std::collections::HashMap;

use worker::*;

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

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
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
    let data: HashMap<String, String> = req.json().await?;
    if let Some(msg) = data.get("message") {
        // return Response::ok(msg);
        let kv = ctx.kv("TIMETABLE_KV")?;
        kv.put("message", msg)?.execute().await?;
        return Response::empty();
    }

    Response::error("Bad Request", 400)
}

pub fn handle_version<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
    Response::ok(version)
}
