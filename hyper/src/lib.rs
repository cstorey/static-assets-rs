use std::task;

use futures::future;
use hyper::{header::CONTENT_TYPE, http, service::Service, Body, Request, Response, StatusCode};
use log::{debug, trace};
use static_assets::Map;

pub use static_assets_macros::static_assets;

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
        trace!("Path: {:?}; tail: {:?}", path, tail);
        let asset = match self.assets.get(tail) {
            Some(asset) => asset,
            None => {
                debug!("No match for path: {:?}", path);
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty());
                return future::ready(resp);
            }
        };

        let resp = Response::builder()
            .header(CONTENT_TYPE, asset.content_type)
            .body(Body::from(asset.content));
        future::ready(resp)
    }
}
