mod assets {
    use static_assets::Map;

    pub(crate) static ASSETS: Map = include!(concat!(env!("OUT_DIR"), "/canary-gen.rs"));
}

#[test]
fn can_fetch_assets_from_build_script_generated_file() {
    let res = assets::ASSETS.get("canary.html");

    assert!(res.is_some(), "Asset canary.html is present");
}
