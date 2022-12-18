#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;

use arma_rs::{arma, Context, Extension};
use synixe_events::{
    arma_server::publish::Publish,
    discord::write::{DiscordContent, DiscordMessage},
    publish,
};
use synixe_proc::events_request;
use tokio::sync::RwLock;

#[macro_use]
extern crate log;

mod background;
mod discord;
mod gear;
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
    if let Err(e) = events_request!(
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
        .command("test_tokio", command_test_tokio)
        .group("gear", gear::group())
        .group("discord", discord::group())
        .finish();
    let ctx_tokio = ext.context();
    std::thread::spawn(move || {
        RUNTIME.block_on(async {
            *CONTEXT.write().await = Some(ctx_tokio);
            if let Err(e) = publish!(
                bootstrap::NC::get().await,
                Publish::Wake {
                    id: SERVER_ID.clone(),
                }
            )
            .await
            {
                error!("failed to publish wake event: {e}");
                panic!("failed to publish wake event: {e}");
            }
            tokio::join!(background::heart(), background::events());
        });
    });
    logger::init(ext.context());
    ext
}

fn command_id() -> String {
    SERVER_ID.clone()
}

fn command_test_tokio() {
    RUNTIME.spawn(async {
        publish!(
            bootstrap::NC::get().await,
            Publish::Heartbeat {
                id: SERVER_ID.clone()
            }
        )
        .await
        .unwrap();
        CONTEXT
            .read()
            .await
            .as_ref()
            .unwrap()
            .callback_null("crate", "test_tokio");
    });
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicBool, AtomicU8},
            Arc,
        },
        time::Duration,
    };

    use rand::{distributions::Alphanumeric, Rng};

    use synixe_events::arma_server::publish;

    #[tokio::test]
    /// Tests all the basic functionality of the extension
    /// 1. The wake event is received
    /// 2. The heartbeat nats event is emitted from the tokio_test command
    /// 3. The heartbeat nats event is emitted from the tokio runtime
    /// 4. The context queues an event from the tokio_test command
    /// 5. The context queues an event from the tokio runtime
    async fn tokio_thread() {
        let key = format!(
            "test_{}",
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect::<String>()
        );
        std::env::set_var("CRATE_SERVER_ID", &key);

        let count = Arc::new(AtomicU8::new(0));

        std::thread::spawn({
            let count = count.clone();
            let key = key.clone();
            move || {
                let nats = nats::connect(
                    std::env::var("NATS_URL").expect("Expected the NATS_URL in the environment"),
                )
                .unwrap();
                let sub = nats.subscribe("synixe.publish.arma_server").unwrap();
                while let Some(msg) = sub.next() {
                    let Ok((data, _)) = synixe_events::parse_data!(msg, publish::Publish) else {
                        continue;
                    };
                    match data {
                        publish::Publish::Wake { id } | publish::Publish::Heartbeat { id } => {
                            if id == key {
                                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        });

        std::thread::sleep(std::time::Duration::from_secs(2));
        let ext = super::init().testing();
        std::thread::sleep(std::time::Duration::from_secs(3));
        unsafe {
            let (out, code) = ext.call("id", None);
            assert_eq!(code, 0);
            assert_eq!(out, key);
            let (_, code) = ext.call("test_tokio", None);
            assert_eq!(code, 0);
        }
        let test_tokio = AtomicBool::new(false);
        let res: arma_rs::Result<(), String> = ext.callback_handler(
            |name, func, data| {
                assert_eq!(name, "crate");
                assert!(data.is_none());
                if func == "beat" {
                    arma_rs::Result::Ok(())
                } else if func == "test_tokio" {
                    test_tokio.store(true, std::sync::atomic::Ordering::SeqCst);
                    arma_rs::Result::Continue
                } else {
                    arma_rs::Result::Err(format!("unexpected callback: {name}::{func}"))
                }
            },
            Duration::from_secs(16),
        );
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(res.is_ok());
        assert!(test_tokio.load(std::sync::atomic::Ordering::SeqCst));

        // Checks for:
        // 1. Wake
        // 2. Heartbeat from tokio_test
        // 3. Heartbeat from the loop
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}
