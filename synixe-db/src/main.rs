#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;
use synixe_events::handler;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

mod handler;

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    bootstrap::logger::init();

    let nats = bootstrap::NC::get().await;

    let sub = nats.subscribe("synixe.db.>").await.unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        handler!(
            msg,
            nats,
            synixe_events::recruiting::db::Request,
            synixe_events::missions::db::Request
        );
    }
}
