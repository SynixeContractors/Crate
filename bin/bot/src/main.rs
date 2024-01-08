#![deny(clippy::pedantic, clippy::unwrap_used)]
#![warn(clippy::nursery, clippy::all)]

use std::sync::Arc;

#[macro_use]
extern crate tracing;

mod bot;
mod cache_http;
mod discord;
mod events;

#[derive(Clone)]
struct ArcCacheAndHttp(Arc<serenity::cache::Cache>, Arc<serenity::http::Http>);

impl ArcCacheAndHttp {
    pub fn as_ref(&self) -> (&Arc<serenity::cache::Cache>, &serenity::http::Http) {
        (&self.0, &self.1)
    }
}

#[tokio::main]
async fn main() {
    bootstrap::logger::init();

    let bot = discord::build().await;

    let cache_and_http = ArcCacheAndHttp(bot.cache.clone(), bot.http.clone());

    cache_http::CacheAndHttp::init(cache_and_http.clone());

    let ((), ()) = tokio::join!(events::start(cache_and_http), discord::start(bot),);
}
