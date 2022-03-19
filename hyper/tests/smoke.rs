use anyhow::{Context, Result};
use axum::{routing::get_service, Router};
use hyper::{
    header::{ETAG, IF_NONE_MATCH},
    Body, Request, StatusCode,
};
use log::warn;
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

#[tokio::test]
async fn should_serve_with_revalidation() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req = Request::builder().uri("/canary.html").body(Body::empty())?;
    let resp = srv.clone().oneshot(req).await.context("Fetch response")?;

    let (parts, _) = resp.into_parts();
    let entity_tag = parts.headers.get(ETAG).expect("some ETag header");

    let mut req_builder = Request::builder().uri("/canary.html");
    req_builder
        .headers_mut()
        .expect("valid builder")
        .insert(IF_NONE_MATCH, entity_tag.clone());
    let req = req_builder.body(Body::empty())?;

    let resp = srv.oneshot(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_MODIFIED);

    Ok(())
}

#[tokio::test]
async fn should_serve_nested_in_axum() -> Result<()> {
    env_logger::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);

    let app = Router::new().nest(
        "/static",
        get_service(srv).handle_error(|error| async move {
            warn!("Error serving request: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal error: {}", error),
            )
        }),
    );

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/static/canary.html")
                .body(Body::empty())?,
        )
        .await
        .unwrap();

    println!("Resp: {:?}", resp);
    assert!(resp.status().is_success());

    let body = hyper::body::to_bytes(resp.into_body()).await?;
    let bodystr = std::str::from_utf8(&body).context("utf8 body")?;

    assert_eq!(bodystr, "<p>Hi!</p>\n");
    Ok(())
}