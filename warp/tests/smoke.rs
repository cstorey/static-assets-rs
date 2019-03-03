#[macro_use]
extern crate static_assets_macros;
extern crate bytes;
extern crate mime;
extern crate static_assets_warp;

use bytes::Bytes;
use std::str::FromStr;

static_assets!(assets, "tests/assets");

#[test]
fn test_response() {
    let filter = static_assets_warp::assets(&assets);

    let value = warp::test::request().path("/canary.html").reply(&filter);
    assert_eq!(value.status(), warp::http::StatusCode::OK);

    assert_eq!(value.body(), &Bytes::from(&b"<p>Hi!</p>\n"[..]));
}

#[test]
fn test_has_content_type() {
    let filter = static_assets_warp::assets(&assets);

    let value = warp::test::request().path("/canary.html").reply(&filter);
    let content_type = value
        .headers()
        .get("content-type")
        .expect("content-type value")
        .to_str()
        .expect("Convert header value to string");
    let parsed = mime::Mime::from_str(content_type).expect("parse content type");
    assert_eq!(
        (parsed.type_(), parsed.subtype()),
        (mime::TEXT, mime::HTML),
        "Content type {:?} (parsed as {:?}) should be text/html",
        content_type,
        parsed
    )
}

#[test]
fn test_has_revalidation() {
    let filter = static_assets_warp::assets(&assets);

    let orig = warp::test::request().path("/canary.html").reply(&filter);
    eprintln!("Orig headers: {:?}", orig.headers());
    let etag = orig.headers().get("etag").expect("entity tag value");

    let revalidated = warp::test::request()
        .path("/canary.html")
        .header("If-None-Match", etag)
        .reply(&filter);

    assert_eq!(revalidated.status(), warp::http::StatusCode::NOT_MODIFIED)
}
