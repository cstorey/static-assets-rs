use anyhow::{Context, Result};
use hyper::{Body, Request};
use tower::ServiceExt;

use static_assets_hyper::{static_assets, StaticService};

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
