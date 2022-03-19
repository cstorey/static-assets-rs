use std::convert::Infallible;

use anyhow::Result;
use hyper::service::make_service_fn;
use hyper::Server;
use static_assets_hyper::{static_assets, StaticService};

static_assets!(ASSETS, "examples/assets");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(StaticService::new(&ASSETS)) });

    let addr = ([127, 0, 0, 1], 8088).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("http://{:?}/index.html", server.local_addr());
    server.await?;

    Ok(())
}
