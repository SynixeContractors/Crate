#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

mod actor;
mod handler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    handler::start().await;
}
