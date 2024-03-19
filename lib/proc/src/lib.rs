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
/// Request a response from a service
/// Timeout: 2 seconds
pub fn events_request_2(item: TokenStream) -> TokenStream {
    request::request(item, 2)
}

#[proc_macro]
/// Request a response from a service
/// Timeout: 5 seconds
pub fn events_request_5(item: TokenStream) -> TokenStream {
    request::request(item, 5)
}

#[proc_macro]
/// Request a response from a service
/// Timeout: 30 seconds
pub fn events_request_30(item: TokenStream) -> TokenStream {
    request::request(item, 30)
}
