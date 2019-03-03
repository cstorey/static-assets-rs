#[macro_use]
extern crate static_assets_macros;
extern crate bytes;
extern crate static_assets_warp;

use bytes::Bytes;

static_assets!(assets, "tests/assets");

#[test]
fn test_response() {
    let filter = static_assets_warp::assets(&assets);

    // Execute `sum` and get the `Extract` back.
    let value = warp::test::request().path("/canary.html").reply(&filter);
    assert_eq!(value.body(), &Bytes::from(&b"<p>Hi!</p>\n"[..]));
}
