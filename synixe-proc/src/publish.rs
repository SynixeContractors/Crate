use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Expr, Result,
};

pub fn publish(item: TokenStream) -> TokenStream {
    let defs = syn::parse_macro_input!(item as Definitions);
    let arms = defs.arms;
    let path = format!(
        "synixe.{}",
        defs.path.to_token_stream().to_string().trim_matches('"')
    );

    let mut publish = Vec::new();
    let mut names = Vec::new();

    for def in arms {
        let attrs = def.attrs;
        let pat = def.name;
        let mut obj = TokenStream::from(quote::quote!(struct));
        obj.extend(TokenStream::from(quote::quote!(#pat)));
        let struct_obj = syn::parse_macro_input!(obj as syn::ItemStruct);
        let name = struct_obj.ident;
        names.push(name.clone());
        publish.push(quote::quote!(
            #(#attrs)*
            #pat
        ));
    }
    let string_names = names
        .clone()
        .into_iter()
        .map(|n| n.to_string().to_lowercase())
        .collect::<Vec<_>>();
    TokenStream::from(quote::quote!(
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        /// An event that expects a response.
        pub enum Publish {
            #(#publish),*
        }
        #[async_trait::async_trait]
        impl crate::Publishable for Publish {
            fn path() -> &'static str {
                #path
            }
            fn name(&self) -> &'static str {
                match self {
                    #(Self::#names { .. } => #string_names),*
                }
            }
        }
    ))
}

struct Definitions {
    path: String,
    arms: Vec<Definition>,
}

impl Parse for Definitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse::<Expr>()?.to_token_stream().to_string();
        let content;
        braced!(content in input);
        let mut arms = Vec::new();
        while !content.is_empty() {
            arms.push(content.parse::<Definition>()?);
        }
        Ok(Self { path, arms })
    }
}

struct Definition {
    attrs: Vec<Attribute>,
    name: Expr,
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            name: input.parse::<Expr>()?,
        })
    }
}
