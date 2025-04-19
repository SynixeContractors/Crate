use synixe_events::{Evokable, discord};

use crate::ArcCacheAndHttp;

mod executions;
mod info;
mod listener;
mod write;

pub async fn start(http: ArcCacheAndHttp) {
    tokio::join!(
        write(http.clone()),
        info(http.clone()),
        executions(http),
        listener::start()
    );
}

async fn write(http: ArcCacheAndHttp) {
    let sub = bootstrap::NC::get()
        .await
        .queue_subscribe(discord::write::Request::path(), "synixe-bot")
        .await
        .expect("Failed to subscribe to write queue");
    while let Some(msg) = sub.next().await {
        write::handle(msg, http.clone()).await;
    }
}

async fn info(http: ArcCacheAndHttp) {
    let sub = bootstrap::NC::get()
        .await
        .queue_subscribe(discord::info::Request::path(), "synixe-bot")
        .await
        .expect("Failed to subscribe to info queue");
    while let Some(msg) = sub.next().await {
        info::handle(msg, http.clone()).await;
    }
}

async fn executions(http: ArcCacheAndHttp) {
    let sub = bootstrap::NC::get()
        .await
        .queue_subscribe(discord::executions::Request::path(), "synixe-bot")
        .await
        .expect("Failed to subscribe to info queue");
    while let Some(msg) = sub.next().await {
        executions::handle(msg, http.clone()).await;
    }
}
