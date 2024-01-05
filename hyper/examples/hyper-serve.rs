use std::net::SocketAddr;

use anyhow::Result;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use static_assets::Map;
use static_assets_hyper::{assets, StaticService};
use tokio::net::TcpListener;

static ASSETS: Map = assets!("examples/assets");

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap_or_default();

    let static_service = StaticService::new(&ASSETS);

    let addr = SocketAddr::new([127, 0, 0, 1].into(), 8088);

    let listener = TcpListener::bind(addr).await?;
    println!("http://{:?}/index.html", listener.local_addr()?);

    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let svc = static_service.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, svc)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
