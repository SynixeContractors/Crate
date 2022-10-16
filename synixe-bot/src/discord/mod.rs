use serenity::prelude::*;

mod handler;
mod slash;

pub async fn build() -> Client {
    // Login with a bot token from the environment
    let token = std::env::var("DISCORD_TOKEN").expect("token");
    Client::builder(
        token,
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS,
    )
    .event_handler(handler::Handler)
    .await
    .expect("Error creating client")
}

pub async fn start(mut client: Client) {
    if let Err(why) = client.start().await {
        error!("start error: {:?}", why);
    }
}
