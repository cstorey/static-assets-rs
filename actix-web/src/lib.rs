extern crate static_assets;
#[macro_use]
extern crate log;

use actix_web::dev::Handler;
use actix_web::{http, HttpRequest, HttpResponse};

use static_assets::Map;

pub struct Static {
    assets: &'static Map<'static>,
}

impl Static {
    pub fn new(assets: &'static Map<'static>) -> Self {
        Static { assets }
    }
}

impl<S> Handler<S> for Static {
    type Result = Result<HttpResponse, actix_web::error::Error>;

    fn handle(&self, req: &HttpRequest<S>) -> Self::Result {
        let tail: String = req
            .match_info()
            .get_decoded("tail")
            .unwrap_or_else(|| "".to_string());
        let path = tail.trim_start_matches('/');

        info!("Path: {:?}; tail: {:?}", req.path(), path);
        let asset = match self.assets.get(&path) {
            Some(asset) => asset,
            None => {
                warn!("No match for path: {:?}", path);
                return Ok(HttpResponse::NotFound().finish());
            }
        };

        let etag = format!(
            "\"{}\"",
            base64::encode_config(asset.digest, base64::URL_SAFE_NO_PAD)
        );

        let not_modified = req
            .headers()
            .get(http::header::IF_NONE_MATCH)
            .and_then(|val| val.to_str().ok())
            .map(|val| val == etag)
            .unwrap_or(false);

        if not_modified {
            Ok(HttpResponse::NotModified().finish())
        } else {
            let resp = HttpResponse::Ok()
                .content_type(asset.content_type)
                .header(http::header::ETAG, etag)
                .body(asset.content);
            Ok(resp)
        }
    }
}
