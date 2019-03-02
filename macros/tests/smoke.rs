#[macro_use]
extern crate static_assets_macros;

use std::collections::BTreeSet;

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

#[test]
fn supports_iterators_non_trivially() {
    let names = assets
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
    for a in assets.iter() {
        for b in assets.iter().filter(|b| a.name != b.name) {
            assert_ne!(a.digest, b.digest, "Digest for {:?} vs {:?}", a, b)
        }
    }
    let res = assets.get("canary.html").expect("asset canary.html");

    assert_eq!(res.content_type, "text/html");
}
