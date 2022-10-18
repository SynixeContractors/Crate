#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use rust_embed::RustEmbed;
use synixe_events::handler;

#[macro_use]
extern crate log;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

mod handler;

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    bootstrap::logger::init();

    // Init NATS connection
    let nats = bootstrap::NC::get().await;

    let sub = nats.subscribe("synixe.executor.>").await.unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        handler!(
            msg,
            nats,
            synixe_events::recruiting::executions::Request,
            synixe_events::missions::executions::Request
        );
    }
}
