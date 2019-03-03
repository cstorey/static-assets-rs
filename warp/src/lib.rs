extern crate warp;
// extern crate http;
extern crate static_assets;

use warp::Filter;

fn asset_maybe(
    assets: &'static static_assets::Map<'static>,
    path: &str,
) -> Result<impl warp::Reply, warp::Rejection> {
    assets
        .get(path)
        .map(|a| warp::reply::with_header(a.content, "content-type", a.content_type))
        .ok_or_else(warp::reject::not_found)
}

pub fn assets(
    assets: &'static static_assets::Map<'static>,
) -> impl warp::Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> {
    warp::path::tail().and_then(move |tail: warp::path::Tail| asset_maybe(assets, tail.as_str()))
}
