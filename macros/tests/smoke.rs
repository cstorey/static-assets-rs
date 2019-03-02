#[macro_use]
extern crate static_assets_macros;

static_assets!(assets, "tests/assets");

#[test]
fn should_lookup_example() {
    let res = assets.get("js/canary.js").expect("asset js/canary.js");

    assert_eq!(res.content, b"console.log(\"Hi\")");
}

#[test]
fn should_have_content_type() {
    let res = assets.get("canary.html").expect("asset canary.html");

    assert_eq!(res.content_type, "text/html");
}
