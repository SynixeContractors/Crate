use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token::FatArrow,
    Attribute, Expr, ItemStruct, Result, TypeParen,
};

pub fn requests(item: TokenStream) -> TokenStream {
    let defs = syn::parse_macro_input!(item as Definitions);
    let arms = defs.arms;
    let path = format!(
        "synixe.{}",
        defs.path.to_token_stream().to_string().trim_matches('"')
    );

    let mut reqs = Vec::new();
    let mut resps = Vec::new();
    let mut names = Vec::new();

    for def in arms {
        let attrs = def.attrs;
        let body = def.body.fields;
        let name = def.body.ident;
        names.push(name.clone());
        let resp = def.resp;
        reqs.push(quote::quote!(
            #(#attrs)*
            #name #body
        ));
        resps.push(quote::quote!(
            #(#attrs)*
            #name #resp
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
        pub enum Request {
            #(#reqs),*
        }
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        /// Response to an event.
        pub enum Response {
            #(#resps),*
        }

        #[async_trait::async_trait]
        impl crate::Evokable for Request {
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
    body: ItemStruct,
    _fat_arrow_token: FatArrow,
    resp: Box<TypeParen>,
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            body: input.parse::<ItemStruct>()?,
            _fat_arrow_token: input.parse::<FatArrow>()?,
            resp: Box::new(input.parse::<TypeParen>()?),
        })
    }
}
