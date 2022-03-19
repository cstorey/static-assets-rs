use anyhow::{Context, Result};
use hyper::{Body, Request, StatusCode};
use tower::ServiceExt;

use static_assets_hyper::{static_assets, StaticService};
use typed_headers::{ContentType, HeaderMapExt};

static_assets!(ASSETS, "../macros/tests/assets");

#[tokio::test]
async fn should_serve_asset_content() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);

    let req = Request::builder().uri("/canary.html").body(Body::empty())?;
    let resp = srv.oneshot(req).await.context("Fetch response")?;

    assert!(resp.status().is_success());

    let (_, body) = resp.into_parts();
    let body = hyper::body::to_bytes(body).await.unwrap();
    let bodystr = std::str::from_utf8(&body).context("utf8 body")?;
    assert_eq!(bodystr, "<p>Hi!</p>\n");

    Ok(())
}

#[tokio::test]
async fn should_serve_404_when_missing() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req = Request::builder()
        .uri("/not-an-asset")
        .body(Body::empty())?;
    let resp = srv.oneshot(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn should_serve_content_type() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req = Request::builder().uri("/canary.html").body(Body::empty())?;
    let resp = srv.oneshot(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    let content_type = parts
        .headers
        .typed_get::<ContentType>()
        .expect("content-type header decode")
        .expect("some content-type header");
    assert_eq!(
        (content_type.type_(), content_type.subtype()),
        (mime::TEXT, mime::HTML)
    );

    Ok(())
}
