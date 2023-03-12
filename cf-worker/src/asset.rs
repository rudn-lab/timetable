use worker::{kv::KvStore, Request, Response, RouteContext};

// Checkout this (issue)[https://github.com/cloudflare/workers-rs/issues/54] for other seriving
// options
pub async fn serve_asset<D>(req: Request, ctx: RouteContext<D>) -> worker::Result<Response> {
    if let Ok(kv) = ctx.kv("__STATIC_CONTENT") {
        let path = req.path();
        let path = path.trim_start_matches('/');
        let value = match kv.get(path).bytes().await? {
            Some(value) => value,
            None => return Response::error("Not Found", 404),
        };
        let mut response = Response::from_bytes(value)?;
        response
            .headers_mut()
            .set("Content-Type", get_mime(path).unwrap_or("text/plain"))?;
        return Ok(response);
    }

    Response::ok("Nothing")
}

fn get_hashed_name(path: &str) -> Option<&'static str> {
    todo!(
        "
    The plan is to loop through all keys and find the one that matches.
    Keys from filename `name.ext` have this format `name.<10-letter-hash>.ext`
    Thankfully, when static files are changed, their old versions are removed from KV,\
        so no duplication occurs.
    "
    )
}

fn get_mime(path: &str) -> Option<&'static str> {
    let ext = if let Some((_, ext)) = path.rsplit_once(".") {
        ext
    } else {
        return None;
    };

    let ct = match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        _ => return None,
    };

    return Some(ct);
}
