use blake2::{Blake2s256, Digest};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeSet;
use std::path::{PathBuf, Path};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("I/O")]
    IO(#[from] std::io::Error),
    #[error("Walking directory tree")]
    WalkDir(#[from] walkdir::Error),
    #[error("Non-utf8 path")]
    NonUtf8Path(PathBuf),
    #[error("Expected directory {0} to contain found file {1}")]
    FoundFileNotInSourceDirectory(PathBuf, PathBuf),
}

struct Input {
    name: syn::Ident,
    path: syn::LitStr,
}
impl Parse for Input {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;
        Ok(Input { name, path })
    }
}
fn root_dir() -> Result<PathBuf, Error> {
    let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        eprintln!("Environment variable $CARGO_MANIFEST_DIR not set, assuming \".\"");
        ".".into()
    });
    Ok(PathBuf::from(base).canonicalize()?)
}

#[proc_macro]
pub fn static_assets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Input { name, path } = parse_macro_input!(input as Input);

    generate(name, path.value().as_ref()).expect("generate").into()
}

fn generate(name: syn::Ident, path: &Path) -> Result<TokenStream, Error> {
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
        static #name : ::static_assets::Map<'static> = ::static_assets::Map{ members: &[#members]};
    );

    Ok(out)
}
