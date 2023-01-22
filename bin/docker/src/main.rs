#[macro_use]
extern crate tracing;

mod handler;
mod listener;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    tokio::join!(listener::start(), handler::start(),);
}
