#![deny(clippy::pedantic, clippy::unwrap_used)]
#![warn(clippy::nursery, clippy::all)]
#![allow(clippy::needless_pass_by_value, clippy::significant_drop_tightening)]

use std::collections::HashMap;

use arma_rs::{arma, Context, Extension};
use synixe_events::discord::write::{DiscordContent, DiscordMessage};
use synixe_proc::events_request_5;
use tokio::sync::RwLock;
use uuid::Uuid;

#[macro_use]
extern crate log;

mod commands;
mod handler;
mod listener;
mod logger;

lazy_static::lazy_static! {
    static ref SERVER_ID: String = std::env::var("CRATE_SERVER_ID").expect("CRATE_SERVER_ID not set");
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");
    pub static ref CONTEXT: RwLock<Option<Context>> = RwLock::new(None);
    pub static ref STEAM_CACHE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

async fn audit(message: String) {
    if let Err(e) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::discord::write,
        Audit {
            message: DiscordMessage {
                content: DiscordContent::Text(message),
                reactions: Vec::new(),
            }
        }
    )
    .await
    {
        error!("failed to send audit message over nats: {}", e);
    }
}

#[arma]
fn init() -> Extension {
    info!("Initializing for server `{}`", *SERVER_ID);
    let ext = Extension::build()
        .command("id", command_id)
        .command("uuid", command_uuid)
        .command("ping", ping)
        .group("campaigns", commands::campaigns::group())
        .group("discord", commands::discord::group())
        .group("garage", commands::garage::group())
        .group("gear", commands::gear::group())
        .group("log", commands::log::group())
        .group("reputation", commands::reputation::group())
        .group("certifications", commands::certifications::group())
        .state(commands::garage::PendingSpawn::default())
        .finish();

    let ctx_tokio = ext.context();
    std::thread::spawn(move || {
        RUNTIME.block_on(async {
            *CONTEXT.write().await = Some(ctx_tokio);
            tokio::join!(handler::start(), listener::start());
        });
    });
    logger::init(ext.context());
    ext
}

fn ping() {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        if events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::campaigns::db,
            Markers {
                campaign: Uuid::default(),
            }
        )
        .await
        .is_ok()
        {
            let _ = context.callback_null("crate", "ping:ok");
        } else {
            let _ = context.callback_null("crate", "ping:err");
        }
    });
}

fn command_id() -> String {
    SERVER_ID.clone()
}

fn command_uuid() -> Uuid {
    Uuid::new_v4()
}
