#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

mod handler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    bootstrap::tracer!("db");
    handler::start().await;
}
