#![allow(clippy::needless_pass_by_value, clippy::significant_drop_tightening)]

use std::{collections::HashMap, sync::LazyLock};

use arma_rs::{Context, Extension, arma};
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

static SERVER_ID: LazyLock<String> =
    LazyLock::new(|| std::env::var("CRATE_SERVER_ID").expect("CRATE_SERVER_ID not set"));
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime")
});
static CONTEXT: LazyLock<RwLock<Option<Context>>> = LazyLock::new(|| RwLock::new(None));
static STEAM_CACHE: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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
        error!("failed to send audit message over nats: {e}");
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
        .group("certifications", commands::certifications::group())
        .group("discord", commands::discord::group())
        .group("fuel", commands::fuel::group())
        .group("garage", commands::garage::group())
        .group("gear", commands::gear::group())
        .group("log", commands::log::group())
        .group("reputation", commands::reputation::group())
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
