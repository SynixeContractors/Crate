#![deny(clippy::pedantic, clippy::unwrap_used)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate tracing;

mod bot;
mod cache_http;
mod discord;
mod events;

type ArcCacheAndHttp = std::sync::Arc<serenity::CacheAndHttp>;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let bot = discord::build().await;

    cache_http::CacheAndHttp::init(bot.cache_and_http.clone());

    let (_, _) = tokio::join!(
        events::start(bot.cache_and_http.clone()),
        discord::start(bot),
    );
}
