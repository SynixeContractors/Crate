use synixe_events::{discord, Evokable};

use crate::Bot;

mod info;
mod listener;
mod write;

pub async fn start(http: Bot) {
    tokio::join!(write(http.clone()), info(http), listener::start());
}

async fn write(http: Bot) {
    let sub = bootstrap::NC::get()
        .await
        .queue_subscribe(discord::write::Request::path(), "synixe-bot")
        .await
        .expect("Failed to subscribe to write queue");
    while let Some(msg) = sub.next().await {
        write::handle(msg, http.clone()).await;
    }
}

async fn info(http: Bot) {
    let sub = bootstrap::NC::get()
        .await
        .queue_subscribe(discord::info::Request::path(), "synixe-bot")
        .await
        .expect("Failed to subscribe to info queue");
    while let Some(msg) = sub.next().await {
        info::handle(msg, http.clone()).await;
    }
}
