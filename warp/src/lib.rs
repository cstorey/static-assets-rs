extern crate base64;
extern crate static_assets;
extern crate warp;
extern crate futures;

use warp::Filter;
use futures::future::Either;

type ETag = Vec<u8>;

fn asset_maybe(
    assets: &'static static_assets::Map<'static>,
    path: &str,
) -> Result<&'static static_assets::Asset<'static>, warp::Rejection> {
    assets.get(path).ok_or_else(warp::reject::not_found)
}

pub fn assets(
    assets: &'static static_assets::Map<'static>,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> {
    warp::path::tail()
        .and_then(move |tail: warp::path::Tail| asset_maybe(assets, tail.as_str()))
        .and(
            warp::header("if-none-match")
                .map(|s: String| base64::decode_config(&s, base64::URL_SAFE).ok())
                .or_else(|_| Ok((None,))),
        )
        .and_then(
            |a: &'static static_assets::Asset<'static>, ifNoneMatch: Option<ETag>| {
                 match ifNoneMatch {
                        Some(ref val) if &**val == a.digest => Either::A(futures::future::ok(warp::reply::with_status(
                            warp::reply(),
                            warp::http::StatusCode::NOT_MODIFIED,
                        ))),
                        _ => {
                            let etag = base64::encode_config(a.digest, base64::URL_SAFE);
                            let resp = warp::reply::with_header(
                                a.content,
                                warp::http::header::CONTENT_TYPE,
                                a.content_type,
                            );
                            Either::B(futures::future::ok(warp::reply::with_header(resp, warp::http::header::ETAG, etag)))
                        }
                    }
            })


}
