use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Expr, ExprStruct, Path, Result, Token,
};

pub fn request(item: TokenStream) -> TokenStream {
    let event = syn::parse_macro_input!(item as Request);
    let path = event.path;
    let nats = event.nats;
    let body = event.body;
    TokenStream::from(quote! {
        async {
            use synixe_events::Evokable;
            let body = #path::Request::#body;
            let path = body.self_path();
            trace!("requesting on {:?}", path);
            let mut trace_body = synixe_events::Wrapper::new(body);
            let response = #nats.request_timeout(
                path,
                synixe_events::serde_json::to_vec(&trace_body).unwrap(),
                std::time::Duration::from_secs(5),
            ).await;
            match response {
                Ok(response) => {
                    Ok(synixe_events::parse_data!(response, #path::Response))
                }
                Err(e) => {
                    error!("Error in request {}: {}", path, e);
                    Err(e.to_string())
                }
            }
        }
    })
}

struct Request {
    nats: Expr,
    _comma1: Token![,],
    path: Path,
    _comma2: Token![,],
    body: ExprStruct,
}

impl Parse for Request {
    fn parse(input: ParseStream) -> Result<Self> {
        let nats = input.parse()?;
        let comma1 = input.parse()?;
        let path = input.parse()?;
        let comma2 = input.parse()?;
        let body = input.parse()?;
        Ok(Self {
            nats,
            _comma1: comma1,
            path,
            _comma2: comma2,
            body,
        })
    }
}
