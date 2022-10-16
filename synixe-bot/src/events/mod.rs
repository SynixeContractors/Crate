use nats::asynk::Connection;
use synixe_events::{discord, Evokable};

use crate::Bot;

mod info;
mod write;

pub async fn start(http: Bot) {
    let nc = nats::asynk::connect(
        std::env::var("NATS_URL").expect("Expected the NATS_URL in the environment"),
    )
    .await
    .unwrap();
    tokio::join!(write(&nc, http.clone()), info(&nc, http),);
}

async fn write(nc: &Connection, http: Bot) {
    let sub = nc
        .queue_subscribe(discord::write::Request::path(), "synixe-bot")
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        write::handle(msg, http.clone()).await;
    }
}

async fn info(nc: &Connection, http: Bot) {
    let sub = nc
        .queue_subscribe(discord::info::Request::path(), "synixe-bot")
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        info::handle(msg, http.clone()).await;
    }
}
