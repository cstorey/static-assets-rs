use std::{env, io::Write, path::PathBuf};

use anyhow::{Context, Result};

use syn::{parse_quote, Ident, Visibility};

pub fn main() -> Result<()> {
    let target_dir = PathBuf::from(env::var("OUT_DIR").expect("$OUT_DIR"));
    let target = target_dir.join("canary-gen.rs");

    let data = static_assets_codegen::generate(
        syn::parse_str::<Visibility>("pub(crate)").context("Parse visibility")?,
        syn::parse_str::<Ident>("ASSETS").context("Parse ident")?,
        "../macros/tests/assets".as_ref(),
    )
    .context("code generation")?;

    let mut tmpf = tempfile::NamedTempFile::new_in(target_dir).context("Creating temp file")?;

    tmpf.write_all(prettyplease::unparse(&parse_quote!(
        #data
    )).as_bytes())
        .with_context(|| format!("Writing content to {:?}", tmpf))?;

    tmpf.persist(&target)
        .with_context(|| format!("Persisting content to {:?}", target))?;

    Ok(())
}
