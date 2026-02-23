use serenity::{
    all::{Context, EventHandler, GatewayIntents, GuildId, Ready},
    async_trait, Client,
};
use synixe_proc::events_request_5;
use tracing::{error, trace};

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("token");
    Client::builder(token, GatewayIntents::all())
        .event_handler(Handler {})
        .await
        .expect("Error creating client")
        .start()
        .await
        .expect("Error starting client");
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild = GuildId::new(700_888_247_928_356_905);
        let members = guild
            .members(&ctx.http, None, None)
            .await
            .expect("Failed to fetch members")
            .into_iter();

        for member in members {
            if member.user.bot {
                continue;
            }
            let id = member.user.id;
            let _ = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankDepositNew {
                    id: Some(
                        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000")
                            .expect("uuid")
                    ),
                    member: id,
                    amount: 3250,
                    reason: "Starting Funds".to_string(),
                }
            )
            .await;
        }

        std::process::exit(0);
    }
}
