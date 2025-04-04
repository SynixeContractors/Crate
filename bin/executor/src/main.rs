use rust_embed::RustEmbed;

#[macro_use]
extern crate tracing;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

mod handler;
mod scheduler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    let mut sched = scheduler::create();
    tokio::join!(handler::start(), sched.start());
}
