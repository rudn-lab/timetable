use worker::{kv::Key, Response, RouteContext};

// Checkout this https://github.com/cloudflare/workers-rs/issues/54 for other seriving options
pub async fn serve_asset<D>(ctx: RouteContext<D>) -> worker::Result<Response> {
    let asset_name = ctx
        .param("asset")
        .map(String::as_str)
        .unwrap_or("index.html");
    if let Ok(kv) = ctx.kv("__STATIC_CONTENT") {
        let response = if let Some(asset) = get_asset_data(&kv, asset_name).await {
            let mut response = Response::from_bytes(asset)?;
            response
                .headers_mut()
                .set("Content-Type", get_mime(asset_name).unwrap_or("text/plain"))?;
            response
        } else {
            let error_data = get_asset_data(&kv, "error.html")
                .await
                .expect("Could not get error page's data");
            let mut resp = Response::from_bytes(error_data)
                .expect("Could not construct error response from bytes");
            resp.headers_mut()
                .set("Content-Type", "text/html")
                .expect("Could not set Content-Type header");
            resp.with_status(404)
        };

        return Ok(response);
    }

    Response::ok("Nothing")
}

trait StaticAssetGiver {
    fn serve(&self, asset: &str) -> Option<Vec<u8>>;
}

async fn get_asset_data(store: &worker::kv::KvStore, asset_name: &str) -> Option<Vec<u8>> {
    if let Some((name, ext)) = asset_name.rsplit_once('.') {
        let keys: Vec<Key> = store.list().execute().await.ok()?.keys;
        for key in keys {
            let (f_name, f_ext) = match &key.name.split('.').collect::<Vec<_>>()[..] {
                &[a, .., b] => (a, b),
                _ => unreachable!("Static files should have a file extension"),
            };

            if f_name == name && f_ext == ext {
                return store.get(&key.name).bytes().await.unwrap_or(None);
            }
        }
    }

    None
}

fn get_mime(path: &str) -> Option<&'static str> {
    let ext = if let Some((_, ext)) = path.rsplit_once('.') {
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

    Some(ct)
}
