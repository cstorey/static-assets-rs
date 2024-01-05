use anyhow::{Context, Result};
use bytes::Bytes;
use headers::{ContentType, HeaderMapExt};
use http_body_util::{BodyExt, Empty};
use hyper::{
    header::{ETAG, IF_NONE_MATCH},
    service::HttpService,
    Request, StatusCode,
};
use static_assets::Map;

use static_assets_hyper::{assets, StaticService};

static ASSETS: Map = assets!("../macros/tests/assets");

#[tokio::test]
async fn should_serve_asset_content() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);

    let req = Request::builder()
        .uri("/canary.html")
        .body(Empty::<Bytes>::new())?;
    let resp = srv.clone().call(req).await.context("Fetch response")?;

    assert!(resp.status().is_success());

    let body: Bytes = resp
        .into_body()
        .collect()
        .await
        .expect("collecting body")
        .to_bytes();
    let bodystr = std::str::from_utf8(&body).context("utf8 body")?;

    assert_eq!(bodystr, "<p>Hi!</p>\n");

    Ok(())
}

#[tokio::test]
async fn should_serve_404_when_missing() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req = Request::builder()
        .uri("/not-an-asset")
        .body(Empty::<Bytes>::new())?;
    let resp = srv.clone().call(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn should_serve_content_type() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv: StaticService = StaticService::new(&ASSETS);
    let req = Request::builder()
        .uri("/canary.html")
        .body(Empty::<Bytes>::new())?;
    let resp = srv.clone().call(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    let content_type: ContentType = parts
        .headers
        .typed_get::<ContentType>()
        .expect("content-type header decode");
    assert_eq!(content_type, ContentType::html(),);

    Ok(())
}

#[tokio::test]
async fn should_serve_not_modified_with_revalidation() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req: Request<_> = Request::builder()
        .uri("/canary.html")
        .body(Empty::<Bytes>::new())?;
    let resp = srv.clone().call(req).await.context("Fetch response")?;

    let (parts, _) = resp.into_parts();
    let entity_tag = parts.headers.get(ETAG).expect("some ETag header");

    let mut req_builder = Request::builder().uri("/canary.html");
    req_builder
        .headers_mut()
        .expect("valid builder")
        .insert(IF_NONE_MATCH, entity_tag.clone());
    let req = req_builder.body(Empty::<Bytes>::new())?;

    let resp = srv.clone().call(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::NOT_MODIFIED);

    Ok(())
}

#[tokio::test]
async fn should_serve_content_with_etag_from_different_resource() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let srv = StaticService::new(&ASSETS);
    let req = Request::builder()
        .uri("/js/canary.js")
        .body(Empty::<Bytes>::new())?;
    let resp = srv.clone().call(req).await.context("Fetch response")?;

    let (parts, _) = resp.into_parts();
    assert_eq!(parts.status, StatusCode::OK);
    let entity_tag = parts.headers.get(ETAG).expect("some ETag header");

    let mut req_builder = Request::builder().uri("/canary.html");
    req_builder
        .headers_mut()
        .expect("valid builder")
        .insert(IF_NONE_MATCH, entity_tag.clone());
    let req = req_builder.body(Empty::<Bytes>::new())?;

    let resp = srv.clone().call(req).await.context("Fetch response")?;
    let (parts, _) = resp.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    Ok(())
}
