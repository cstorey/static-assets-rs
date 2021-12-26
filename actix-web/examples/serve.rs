extern crate static_assets;
#[macro_use]
extern crate static_assets_macros;
extern crate actix_web;
extern crate env_logger;
extern crate static_assets_actix;

use actix_web::middleware::Logger;
use actix_web::{http, web, App, HttpResponse, HttpServer};

use static_assets_actix::Static;

static_assets!(ASSETS, "examples/assets");

fn index() -> HttpResponse {
    HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/index.html")
        .finish()
}

#[actix_rt::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init();

    let s = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .default_service(Static::new(&ASSETS))
            .wrap(Logger::default())
    })
    .bind("0.0.0.0:8088")
    .unwrap();
    println!("{:?}", s.addrs());
    s.run().await?;
    Ok(())
}
