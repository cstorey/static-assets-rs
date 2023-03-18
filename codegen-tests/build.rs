use std::{env, path::PathBuf};

use anyhow::Result;

pub fn main() -> Result<()> {
    let target_dir = PathBuf::from(env::var("OUT_DIR").expect("$OUT_DIR"));
    let target = target_dir.join("canary-gen.rs");

    static_assets_codegen::generate_to_file("../macros/tests/assets".as_ref(), target)?;

    Ok(())
}
