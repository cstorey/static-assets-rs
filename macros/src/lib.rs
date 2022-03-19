use blake2::{Blake2s256, Digest};
use failure::*;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::BTreeSet;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};

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
    let input = parse_macro_input!(input as Input);

    generate(input).expect("generate").into()
}

fn generate(input: Input) -> Result<TokenStream, Error> {
    let Input { name, path } = input;

    let dir = root_dir()?.join(path.value());

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
            .ok_or_else(|| failure::err_msg(format!("Path for {:?}", path)))?;
        let name = path
            .strip_prefix(&dir)
            .context("Removing path prefix")?
            .to_str()
            .ok_or_else(|| failure::err_msg(format!("Path for {:?}", path)))?;

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
