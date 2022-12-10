#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

#[macro_use]
extern crate log;

mod cache_http;
mod discord;
mod events;

type Bot = std::sync::Arc<serenity::CacheAndHttp>;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let bot = discord::build().await;

    cache_http::Bot::init(bot.cache_and_http.clone());

    let (_, _) = tokio::join!(
        events::start(bot.cache_and_http.clone()),
        discord::start(bot),
    );
}
