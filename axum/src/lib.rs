use axum::{
    body::Body,
    debug_handler,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{
    alphabet::URL_SAFE,
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    Engine,
};
use hyper::{
    header::{CONTENT_TYPE, ETAG, IF_NONE_MATCH},
    HeaderMap, StatusCode,
};
use static_assets::Map;
pub use static_assets_macros::assets;
use tracing::{debug, error};

const ETAG_STRING_SIZE: usize = 45;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("http")]
    Http(#[from] axum::http::Error),
}

pub fn assets_router(assets: &'static Map<'static>) -> Router {
    let mut rt = Router::new();

    for asset in assets.iter() {
        let path = format!("/{}", asset.name);
        debug!(?path, "adding asset");
        rt = rt.route(&path, get(get_asset).with_state(asset));
    }

    rt
}

#[debug_handler]
async fn get_asset(
    request_headers: HeaderMap,
    asset: State<static_assets::Asset<'static>>,
) -> Result<impl IntoResponse, Error> {
    let mut buf: [u8; 45] = [0u8; ETAG_STRING_SIZE];
    let etag = encode_etag(&mut buf, &asset);

    let not_modified = request_headers
        .get(IF_NONE_MATCH)
        .and_then(|val| val.to_str().ok())
        .map(|val| val == etag)
        .unwrap_or(false);

    if not_modified {
        let resp = Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::default())?;
        return Ok(resp);
    }

    let resp = Response::builder()
        .header(CONTENT_TYPE, asset.content_type)
        .header(ETAG, etag)
        .body(Body::from(asset.content))?;

    Ok(resp)
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        error!(error=%self, "Error handlng request");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal service error").into_response()
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
