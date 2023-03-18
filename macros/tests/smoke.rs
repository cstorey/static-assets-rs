use std::collections::BTreeSet;

use static_assets::Map;
use static_assets_macros::assets;

static ASSETS: Map = assets!("tests/assets");
pub(crate) static PUB_CRATE_ASSETS: Map = assets!("tests/assets");

#[test]
fn should_lookup_example() {
    let res = ASSETS.get("js/canary.js").expect("asset js/canary.js");

    assert_eq!(res.content, b"console.log(\"Hi\")");
}

#[test]
fn should_lookup_example_from_pub_crate() {
    let res = PUB_CRATE_ASSETS
        .get("js/canary.js")
        .expect("asset js/canary.js");

    assert_eq!(res.content, b"console.log(\"Hi\")");
}

#[test]
fn should_have_content_type() {
    let res = ASSETS.get("canary.html").expect("asset canary.html");

    assert_eq!(res.content_type, "text/html");
}

#[test]
fn supports_iterators_non_trivially() {
    let names = ASSETS
        .iter()
        .map(|a| a.name.to_string())
        .collect::<BTreeSet<_>>();

    assert!(
        names.contains("canary.html"),
        "All names: {:?}; contains canary.html",
        names
    );
}

#[test]
fn should_have_relatively_unique_digests() {
    for a in ASSETS.iter() {
        for b in ASSETS.iter().filter(|b| a.name != b.name) {
            assert_ne!(a.digest, b.digest, "Digest for {:?} vs {:?}", a, b)
        }
    }
    let res = ASSETS.get("canary.html").expect("asset canary.html");

    assert_eq!(res.content_type, "text/html");
}
