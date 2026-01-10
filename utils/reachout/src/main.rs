use serenity::{
    Client, all::{Context, EventHandler, GatewayIntents, GuildId, Ready, UserId}, async_trait
};

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

        // let mut members = GuildId::new(700888247928356905)
        //     .members(&ctx.http, None, None)
        //     .await
        //     .expect("Failed to get members");
        // members.sort_by_key(|m| m.joined_at);
        // println!("Total members: {}", members.len());
        // let mut keep = false;
        // members.retain(|m| {
        //     if m.user.id == UserId::new(188019722028187659) {
        //         keep = true;
        //         false
        //     } else {
        //         keep
        //     }
        // });
        let members = vec![
            UserId::new(307524009854107648)
        ];

        for member in members {
            let Ok(dm) = member.user.create_dm_channel(&ctx).await else {
                println!("Failed to create DM channel with {member}");
                continue;
            };
            if let Err(e) = dm.say(&ctx, r#"# Synixe 2026

Synixe is starting off 2026 with a gear reset, an exciting new campaign, new mod and persistence features, and all new factions!

## Gear Reset

With changes to our gear mods, we're resetting all gear and bank balances. New year, new Synixe. It's the perfect time for a fresh start! The cost of specialist weapons has been balanced, with the company paying for a chunk of the cost. Getting certs and running useful support roles has never been more affordable!

## New Campaign: Nur al-Sahra

Join us in the desert river valley of Wahat Nura Province in Takistan. Resupplies are few and far between, and success depends on our ability to adapt to supply shortages, keep the civilians on our side, handle shifting hostiles, and command local Takistani forces.

## New Factions

Thanks to the hard work of several community members, we're excited to introduce our new faction mod. Contracts in 2026 will put us in new and challenging situations. Enemies are well equipped with a variety of weapons, gear, and vehicles. No longer will we constantly be fighting the same few insurgent and militia groups.

Synixe plays every Saturday and Sunday at <t:1768078800:t>. The Nur al-Sahra campaign begins <t:1768078800:f> in <#700888805137318039>. We can't wait to see you there!"#).await {
                println!("Failed to send message to {member}: {e}");
                continue;
            }
            println!("Sent message to {member}");
            std::thread::sleep(std::time::Duration::from_secs(5));
        }

        std::process::exit(0);
    }
}
