use static_assets_codegen::generate;
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

#[proc_macro]
pub fn static_assets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Input { name, path } = parse_macro_input!(input as Input);

    generate(name, path.value().as_ref())
        .expect("generate")
        .into()
}

