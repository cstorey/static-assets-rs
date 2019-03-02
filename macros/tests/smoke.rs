#[macro_use]
extern crate static_assets_macros;

static_assets!(assets, "tests/assets");

#[test]
fn should_lookup_example() {
    let res = assets.get("js/canary.js").expect("asset js/canary.js");

    assert_eq!(res.content, b"console.log(\"Hi\")");
}
