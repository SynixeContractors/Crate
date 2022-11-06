#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use arma_rs::{arma, Extension};
use synixe_events::{arma_server::publish::Publish, publish};

#[macro_use]
extern crate log;

mod logger;

lazy_static::lazy_static! {
    static ref SERVER_ID: String = std::env::var("CRATE_SERVER_ID").expect("CRATE_SERVER_ID not set");
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to initialize tokio runtime");
}

#[arma]
fn init() -> Extension {
    info!("Initializing for server `{}`", *SERVER_ID);
    let ext = Extension::build().command("id", command_id).finish();
    std::thread::spawn(move || {
        RUNTIME.block_on(async {
            publish!(
                bootstrap::NC::get().await,
                Publish::Wake {
                    id: SERVER_ID.clone(),
                }
            )
            .await
            .unwrap();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(15));
                if let Err(e) = publish!(
                    bootstrap::NC::get().await,
                    Publish::Heartbeat {
                        id: SERVER_ID.clone(),
                    }
                )
                .await
                {
                    error!("failed to publish heartbeat: {}", e);
                }
            }
        });
    });
    logger::init(ext.context());
    ext
}

fn command_id() -> String {
    SERVER_ID.clone()
}
