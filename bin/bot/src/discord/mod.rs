use std::sync::atomic::AtomicU32;

use serenity::prelude::*;

mod handler;
pub mod interaction;
mod menu;
mod slash;
mod utils;

pub async fn build() -> Client {
    // Login with a bot token from the environment
    let token = std::env::var("DISCORD_TOKEN").expect("token");
    Client::builder(token, GatewayIntents::all())
        .event_handler(handler::Handler {
            brain: handler::Brain::new(),
            subcon_counter: AtomicU32::new(0),
        })
        .await
        .expect("Error creating client")
}

pub async fn start(mut client: Client) {
    if let Err(why) = client.start().await {
        error!("start error: {:?}", why);
    }
}
