#[macro_use]
extern crate tracing;

mod listener;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    listener::start().await;
}
