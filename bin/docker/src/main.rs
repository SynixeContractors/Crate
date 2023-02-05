#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate tracing;

mod handler;
mod listener;

lazy_static::lazy_static! {
    static ref DOCKER_SERVER: String = std::env::var("CRATE_DOCKER_SERVER").expect("CRATE_DOCKER_SERVER not set");
}

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    info!("Starting docker server {}", *DOCKER_SERVER);
    tokio::join!(listener::start(), handler::start());
}
