#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use proc_macro::TokenStream;

mod publish;
mod request;
mod requests;

#[proc_macro]
pub fn events_requests(item: TokenStream) -> TokenStream {
    requests::requests(item)
}

#[proc_macro]
pub fn events_publish(item: TokenStream) -> TokenStream {
    publish::publish(item)
}

#[proc_macro]
pub fn events_request(item: TokenStream) -> TokenStream {
    request::request(item)
}
