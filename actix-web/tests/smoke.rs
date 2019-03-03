extern crate static_assets_actix;
#[macro_use]
extern crate static_assets_macros;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate mime;

use actix_web::{http, test, App, HttpMessage};
use static_assets_actix::Static;

static_assets!(assets, "../macros/tests/assets");

#[test]
fn should_serve_asset_content() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = test::TestServer::with_factory(|| {
        info!("Build app");
        App::new().handler("/s/", Static::new(&assets))
    });
    let req = srv
        .client(http::Method::GET, "/s/canary.html")
        .finish()
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());

    let bytes = srv.execute(response.body()).unwrap();
    let body = String::from_utf8_lossy(&bytes);
    assert_eq!(body, "<p>Hi!</p>\n");
}

#[test]
fn should_serve_404() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = test::TestServer::with_factory(|| {
        info!("Build app");
        App::new().handler("/s/", Static::new(&assets))
    });
    let req = srv
        .client(http::Method::GET, "/s/garbage")
        .finish()
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert_eq!(response.status(), 404);
}

#[test]
fn should_serve_content_type() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = test::TestServer::with_factory(|| {
        info!("Build app");
        App::new().handler("/s/", Static::new(&assets))
    });
    let req = srv
        .client(http::Method::GET, "/s/canary.html")
        .finish()
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    let content_type = response
        .mime_type()
        .expect("mime type")
        .expect("some mime type");
    assert_eq!(
        (content_type.type_(), content_type.subtype()),
        (mime::TEXT, mime::HTML)
    );
}

#[test]
fn should_serve_with_revalidation() {
    env_logger::try_init().unwrap_or_default();

    let mut srv = test::TestServer::with_factory(|| {
        info!("Build app");
        App::new().handler("/s/", Static::new(&assets))
    });
    let req = srv
        .client(http::Method::GET, "/s/canary.html")
        .finish()
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    let etag = response
        .headers()
        .get(actix_web::http::header::ETAG)
        .expect("ETag response header");

    let req = srv
        .client(http::Method::GET, "/s/canary.html")
        .set_header(actix_web::http::header::IF_NONE_MATCH, etag.clone())
        .finish()
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert_eq!(response.status(), actix_web::http::StatusCode::NOT_MODIFIED);
}
