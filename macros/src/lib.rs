use static_assets_codegen::generate;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr};

struct Input {
    path: syn::LitStr,
}
impl Parse for Input {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let path: LitStr = input.parse()?;
        Ok(Input { path })
    }
}

#[proc_macro]
pub fn assets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Input { path } = parse_macro_input!(input as Input);

    generate(path.value().as_ref()).expect("generate").into()
}
