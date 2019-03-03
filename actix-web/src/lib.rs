extern crate static_assets;
#[macro_use]
extern crate log;

use actix_web::dev::Handler;
use actix_web::{HttpRequest, HttpResponse};

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

        if let Some(asset) = self.assets.get(&path) {
            let resp = HttpResponse::Ok()
                .content_type(asset.content_type)
                .body(asset.content);
            Ok(resp)
        } else {
            warn!("No match for path: {:?}", path);
            Ok(HttpResponse::NotFound().finish())
        }
    }
}
