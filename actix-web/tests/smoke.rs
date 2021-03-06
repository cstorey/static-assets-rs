extern crate static_assets_actix;
#[macro_use]
extern crate static_assets_macros;
extern crate env_logger;
extern crate log;
extern crate mime;

use actix_rt;
use actix_service::Service;
use actix_web::{http, test};
use static_assets_actix::Static;

static_assets!(ASSETS, "../macros/tests/assets");

#[actix_rt::test]
async fn should_serve_asset_content() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = Static::new(&ASSETS);

    let req = test::TestRequest::with_uri("/canary.html");
    let resp = srv.call(req.to_srv_request()).await.expect("response");
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let text = String::from_utf8_lossy(&body);
    assert_eq!(text, "<p>Hi!</p>\n");
}

#[actix_rt::test]
async fn should_serve_404() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = Static::new(&ASSETS);
    let req = test::TestRequest::with_uri("/garbage");
    let resp = srv.call(req.to_srv_request()).await.expect("response");
    assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
}

#[cfg(never)]
#[actix_rt::test]
async fn should_serve_content_type() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = Static::new(&ASSETS);

    let req = test::TestRequest::with_uri("/canary.html");
    let resp = srv.call(req.to_srv_request()).await.expect("response");
    let content_type = resp
        .mime_type()
        .expect("mime type")
        .expect("some mime type");
    assert_eq!(
        (content_type.type_(), content_type.subtype()),
        (mime::TEXT, mime::HTML)
    );
}

#[actix_rt::test]
async fn should_serve_with_revalidation() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = Static::new(&ASSETS);

    let req = test::TestRequest::with_uri("/canary.html");
    let resp = srv.call(req.to_srv_request()).await.expect("response");
    let etag = resp
        .headers()
        .get(actix_web::http::header::ETAG)
        .expect("ETag response header");

    let req = test::TestRequest::with_uri("/canary.html")
        .header(http::header::IF_NONE_MATCH, etag.clone());
    let resp = srv.call(req.to_srv_request()).await.expect("response");
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_MODIFIED);
}
