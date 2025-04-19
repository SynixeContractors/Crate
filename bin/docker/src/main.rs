use std::sync::LazyLock;

#[macro_use]
extern crate tracing;

mod handler;
mod listener;

static DOCKER_SERVER: LazyLock<String> =
    LazyLock::new(|| std::env::var("CRATE_DOCKER_SERVER").expect("CRATE_DOCKER_SERVER not set"));

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    info!("Starting docker server {}", *DOCKER_SERVER);
    tokio::join!(listener::start(), handler::start());
}
