#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use opentelemetry::sdk::propagation::TraceContextPropagator;

#[macro_use]
extern crate log;

mod discord;
mod events;

type Bot = std::sync::Arc<serenity::CacheAndHttp>;

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    bootstrap::logger::init();

    let bot = discord::build().await;

    let (_, _) = tokio::join!(
        events::start(bot.cache_and_http.clone()),
        discord::start(bot),
    );
}
