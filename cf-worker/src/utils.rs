use cfg_if::cfg_if;
use worker::{Request, Response, Result, RouteContext};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub async fn auth<D>(req: &Request, ctx: &RouteContext<D>) -> Option<Result<Response>> {
    if let Ok(is_valid) = validate_token(req, ctx).await {
        if !is_valid {
            return Some(Response::error("Unauthorized", 401));
        }
    } else {
        return Some(Response::error("Could not validate auth token", 500));
    }

    None
}

pub async fn validate_token<D>(req: &Request, ctx: &RouteContext<D>) -> Result<bool> {
    let kv = ctx.kv("TIMETABLE_KV")?;
    let valid_token = kv
        .get("WORKER_AUTH")
        .text()
        .await?
        .ok_or("No auth token in KV")?;
    let got_token = req
        .headers()
        .get("Auth-Token")?
        .ok_or("No header 'Auth-Token'")?;

    Ok(valid_token == got_token)
}
