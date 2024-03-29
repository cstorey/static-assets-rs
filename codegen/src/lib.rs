use std::collections::BTreeSet;
use std::fs::read_to_string;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use blake2::{Blake2s256, Digest};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O")]
    IO(#[from] std::io::Error),
    #[error("Walking directory tree")]
    WalkDir(#[from] walkdir::Error),
    #[error("Non-utf8 path")]
    NonUtf8Path(PathBuf),
    #[error("Expected directory {0} to contain found file {1}")]
    FoundFileNotInSourceDirectory(PathBuf, PathBuf),
    #[error("Cannot find parent directory for target: {0}")]
    NoParentDirectory(PathBuf),
    #[error("Persisting temporary file")]
    PersistTempFile(#[from] tempfile::PersistError),
}

fn root_dir() -> Result<PathBuf, Error> {
    let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        eprintln!("Environment variable $CARGO_MANIFEST_DIR not set, assuming \".\"");
        ".".into()
    });
    Ok(PathBuf::from(base).canonicalize()?)
}

pub fn generate(path: &Path) -> Result<TokenStream, Error> {
    let dir = root_dir()?.join(path);

    let mut files = BTreeSet::new();
    for entry in walkdir::WalkDir::new(&dir) {
        let entry = entry?;

        if entry.file_type().is_file() {
            let name = entry.path().to_path_buf();
            files.insert(name);
        }
    }

    let mut members = TokenStream::new();
    for path in files {
        let pathname = path
            .to_str()
            .ok_or_else(|| Error::NonUtf8Path(path.to_owned()))?;
        let name = path
            .strip_prefix(&dir)
            .map_err(|_| Error::FoundFileNotInSourceDirectory(dir.to_owned(), path.to_owned()))?
            .to_str()
            .ok_or_else(|| Error::NonUtf8Path(path.to_owned()))?;

        let content_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        let mut hasher = Blake2s256::default();
        hasher.update(std::fs::read(&path)?);
        let digest_bytes = hasher
            .finalize()
            .iter()
            .map(|b| quote!(#b,))
            .collect::<TokenStream>();

        let asset = quote!(::static_assets::Asset {
            name: #name,
            content: include_bytes!(#pathname),
            content_type: #content_type,
            digest: &[#digest_bytes],
        });

        quote!(#asset,).to_tokens(&mut members)
    }

    let out = quote!(
       ::static_assets::Map{ members: &[#members]}
    );

    Ok(out)
}

pub fn generate_to_file(assets_path: &std::path::Path, target: PathBuf) -> Result<(), Error> {
    let content = generate(assets_path)?;

    write_file_if_changed(&target, &format!("{}", content))?;

    Ok(())
}

fn write_file_if_changed(target: &Path, content: &str) -> Result<(), Error> {
    match read_to_string(target) {
        Ok(existing) => {
            if existing == content {
                return Ok(());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(Error::from(error)),
    };

    let dir = target
        .parent()
        .ok_or_else(|| Error::NoParentDirectory(target.into()))?;

    let mut tmpf = tempfile::NamedTempFile::new_in(dir)?;
    tmpf.write_all(content.as_bytes())?;
    tmpf.persist(target)?;
    Ok(())
}
