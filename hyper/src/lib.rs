use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    Engine,
};
use bytes::Bytes;
use futures::future;
use http_body_util::Full;
use hyper::{
    header::{CONTENT_TYPE, ETAG},
    http,
    service::Service,
    Request, Response, StatusCode,
};
use static_assets::Map;
use tracing::{debug, trace};

pub use static_assets_macros::assets;

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

impl<B: hyper::body::Body> Service<Request<B>> for StaticService {
    // TODO: use UnsyncBoxBody<Bytes, Error>, a-la: https://docs.rs/tower-http/latest/tower_http/services/fs/struct.ServeFileSystemResponseBody.html ?
    type Response = Response<Full<Bytes>>;

    type Error = http::Error;

    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<B>) -> Self::Future {
        let path = req.uri().path();
        let tail = path.strip_prefix('/').unwrap_or(path);
        trace!(?path, ?tail, "Paths");
        let asset = match self.assets.get(tail) {
            Some(asset) => asset,
            None => {
                debug!(?path, "No match for path");
                let resp = Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::default());
                return future::ready(resp);
            }
        };

        let mut buf = [0u8; ETAG_STRING_SIZE];
        let etag = encode_etag(&mut buf, asset);

        let not_modified = req
            .headers()
            .get(http::header::IF_NONE_MATCH)
            .and_then(|val| val.to_str().ok())
            .map(|val| val == etag)
            .unwrap_or(false);

        if not_modified {
            let resp = Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Full::default());
            return future::ready(resp);
        }

        let resp = Response::builder()
            .header(CONTENT_TYPE, asset.content_type)
            .header(ETAG, etag)
            .body(Full::from(asset.content));
        future::ready(resp)
    }
}

fn encode_etag<'a>(buf: &'a mut [u8; ETAG_STRING_SIZE], asset: &static_assets::Asset) -> &'a str {
    const BASE64_ENGINE: GeneralPurpose = GeneralPurpose::new(&URL_SAFE, NO_PAD);
    let mut off = 0;
    buf[off] = b'"';
    off += 1;
    off += BASE64_ENGINE
        .encode_slice(asset.digest, &mut buf[off..])
        .unwrap();
    buf[off] = b'"';
    off += 1;
    std::str::from_utf8(&buf[..off]).expect("Should only generate ASCII")
}
