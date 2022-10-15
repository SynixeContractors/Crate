#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use proc_macro::TokenStream;

mod events;

#[proc_macro]
pub fn events_requests(item: TokenStream) -> TokenStream {
    events::requests(item)
}
