use std::{env, path::PathBuf};

use anyhow::{Context, Result};

use syn::{Ident, Visibility};

pub fn main() -> Result<()> {
    let target_dir = PathBuf::from(env::var("OUT_DIR").expect("$OUT_DIR"));
    let target = target_dir.join("canary-gen.rs");

    static_assets_codegen::generate_to_file(
        syn::parse_str::<Visibility>("pub(crate)").context("Parse visibility")?,
        syn::parse_str::<Ident>("ASSETS").context("Parse ident")?,
        "../macros/tests/assets".as_ref(),
        target,
    )?;

    Ok(())
}

