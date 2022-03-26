use std::task;

use futures::future;
use hyper::{
    header::{CONTENT_TYPE, ETAG},
    http,
    service::Service,
    Body, Request, Response, StatusCode,
};
use static_assets::Map;
use tracing::{debug, trace};

pub use static_assets_macros::static_assets;

const ETAG_STRING_SIZE: usize = 45;

#[derive(Clone)]
pub struct StaticService {
    assets: &'static Map<'static>,
}

impl StaticService {
    pub fn new(assets: &'static Map<'static>) -> Self {
        Self { assets }
    }
}

impl Service<Request<Body>> for StaticService {
    type Response = Response<Body>;

    type Error = http::Error;

    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req.uri().path();
        let tail = path.strip_prefix('/').unwrap_or(path);
        trace!(?path, ?tail, "Paths");
        let asset = match self.assets.get(tail) {
            Some(asset) => asset,
            None => {
                debug!(?path, "No match for path");
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty());
                return future::ready(resp);
            }
        };

        let etag = {
            let mut buf = String::with_capacity(ETAG_STRING_SIZE);
            buf.push('"');
            base64::encode_config_buf(asset.digest, base64::URL_SAFE_NO_PAD, &mut buf);
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
            let resp = Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty());
            return future::ready(resp);
        }

        let resp = Response::builder()
            .header(CONTENT_TYPE, asset.content_type)
            .header(ETAG, etag)
            .body(Body::from(asset.content));
        future::ready(resp)
    }
}
