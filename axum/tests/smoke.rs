use anyhow::{Context, Result};
use axum::{body::Body, routing::get_service, Router};
use headers::{ContentType, HeaderMapExt};
use http_body_util::BodyExt;
use hyper::{
    header::{ETAG, IF_NONE_MATCH},
    Request, StatusCode,
};
use static_assets::Map;
use tower::ServiceExt;
use tracing::warn;

use static_assets_axum::{assets, assets_router};

static ASSETS: Map = assets!("../macros/tests/assets");

#[tokio::test]
async fn should_serve_asset_content() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);

    let req = Request::builder().uri("/canary.html").body(Body::empty())?;
    let resp = srv.oneshot(req).await.context("Fetch response")?;

    assert!(resp.status().is_success());

    let body = resp.into_body().collect().await?.to_bytes();
    assert_eq!(body, "<p>Hi!</p>\n");

    Ok(())
}

#[tokio::test]
async fn should_serve_404_when_missing() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);
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
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);
    let req = Request::builder().uri("/canary.html").body(Body::empty())?;
    let resp = srv.oneshot(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    let content_type: ContentType = parts
        .headers
        // Waiting on typed-headers to upgrade to http 1.0
        .typed_get::<ContentType>()
        .expect("content-type header decode");
    assert_eq!(content_type, ContentType::html(),);

    Ok(())
}

#[tokio::test]
async fn should_serve_not_modified_with_revalidation() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);
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
async fn should_serve_content_with_etag_from_different_resource() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);
    let req = Request::builder()
        .uri("/js/canary.js")
        .body(Body::empty())?;
    let resp = srv.clone().oneshot(req).await.context("Fetch response")?;

    let (parts, _) = resp.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let entity_tag = parts.headers.get(ETAG).expect("some ETag header");

    let mut req_builder = Request::builder().uri("/canary.html");
    req_builder
        .headers_mut()
        .expect("valid builder")
        .insert(IF_NONE_MATCH, entity_tag.clone());
    let req = req_builder.body(Body::empty())?;

    let resp = srv.oneshot(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn should_serve_nested_in_axum() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = assets_router(&ASSETS);

    let app = Router::new().nest_service(
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

    let body = resp
        .into_body()
        .collect()
        .await
        .expect("collecting body")
        .to_bytes();
    let bodystr = std::str::from_utf8(&body).context("utf8 body")?;

    assert_eq!(bodystr, "<p>Hi!</p>\n");
    Ok(())
}
