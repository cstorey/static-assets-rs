extern crate static_assets;
#[macro_use]
extern crate log;

use std::task::{Context, Poll};

use actix_service::{Service, ServiceFactory};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, HttpRequest, HttpResponse};
use futures::future::{ok, ready, Ready};

use static_assets::Map;

const ETAG_STRING_SIZE: usize = 45;

#[derive(Clone)]
pub struct Static {
    assets: &'static Map<'static>,
}

impl Static {
    pub fn new(assets: &'static Map<'static>) -> Self {
        Static { assets }
    }
}

impl Static {
    fn handle(&self, req: HttpRequest) -> Result<HttpResponse, actix_web::error::Error> {
        let tail = req.match_info().unprocessed();
        let path = tail.trim_start_matches('/');

        trace!("Path: {:?}; tail: {:?}", req.path(), path);
        let asset = match self.assets.get(&path) {
            Some(asset) => asset,
            None => {
                debug!("No match for path: {:?}", path);
                return Ok(HttpResponse::NotFound().finish());
            }
        };

        let etag = {
            let mut buf = String::with_capacity(ETAG_STRING_SIZE);
            buf.push('"');
            buf.push_str(&base64::encode_config(
                asset.digest,
                base64::URL_SAFE_NO_PAD,
            ));
            buf.push('"');
            buf
        };

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
impl Service for Static {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = actix_web::error::Error;
    type Future = Ready<Result<ServiceResponse, actix_web::error::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let (httpreq, _) = req.into_parts();
        ready(
            self.handle(httpreq.clone())
                .map(|rsp| ServiceResponse::new(httpreq, rsp)),
        )
    }
}

impl ServiceFactory for Static {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = actix_web::error::Error;
    type Future = Ready<Result<Self, actix_web::error::Error>>;
    type Service = Self;
    type Config = ();
    type InitError = actix_web::error::Error;

    fn new_service(&self, _cfg: Self::Config) -> Self::Future {
        ok(self.clone())
    }
}
