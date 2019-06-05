extern crate static_assets;
#[macro_use]
extern crate static_assets_macros;
extern crate actix_web;
extern crate env_logger;
extern crate static_assets_actix;

use actix_web::{http, server, App, HttpRequest, HttpResponse};

use static_assets_actix::Static;

static_assets!(ASSETS, "examples/assets");

fn index(_req: &HttpRequest) -> HttpResponse {
    HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/index.html")
        .finish()
}

fn main() {
    env_logger::init();

    let s = server::new(|| {
        App::new()
            .resource("/", |r| r.f(index))
            .handler("/", Static::new(&ASSETS))
    })
    .bind("0.0.0.0:8088")
    .unwrap();
    println!("{:?}", s.addrs());
    s.run();
}
