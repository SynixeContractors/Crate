#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use rust_embed::RustEmbed;

#[macro_use]
extern crate tracing;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

mod handler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    handler::start().await;
}
