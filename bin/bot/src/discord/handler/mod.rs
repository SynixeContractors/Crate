use rand::Rng;
use serenity::{async_trait, model::prelude::*, prelude::*};
use synixe_events::{discord::publish::Publish, publish};
use synixe_meta::discord::{
    channel::{BOT, FINANCIALS, LOBBY, LOOKING_TO_PLAY, OFFTOPIC, ONTOPIC, STAFF},
    GUILD,
};
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::{
    bot::Bot,
    discord::menu::{
        missions::{MENU_AAR_IDS, MENU_AAR_PAY},
        recruiting::MENU_RECRUITING_REPLY,
    },
};

pub use self::brain::Brain;

use super::{menu, slash};

mod brain;
mod missions;

pub struct Handler {
    pub brain: Option<Brain>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        Bot::init(ctx.shard.into());

        if let Err(e) =
            GuildId::set_application_commands(&synixe_meta::discord::GUILD, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| menu::missions::aar_ids(command))
                    .create_application_command(|command| menu::missions::aar_pay(command))
                    .create_application_command(|command| menu::recruiting::reply(command))
                    .create_application_command(|command| slash::bank::register(command))
                    .create_application_command(|command| slash::certifications::register(command))
                    .create_application_command(|command| slash::docker::register(command))
                    .create_application_command(|command| slash::garage::register(command))
                    .create_application_command(|command| slash::meme::register(command))
                    .create_application_command(|command| slash::missions::register(command))
                    .create_application_command(|command| slash::reputation::register(command))
                    .create_application_command(|command| slash::reset::register(command))
                    .create_application_command(|command| slash::schedule::register(command))
            })
            .await
        {
            error!("Cannot register slash commands: {}", e);
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) = match interaction {
            Interaction::ApplicationCommand(command) => {
                debug!("matching command: {:?}", command.data.name.as_str());
                match command.data.name.as_str() {
                    "bank" => slash::bank::run(&ctx, &command).await,
                    "certifications" => slash::certifications::run(&ctx, &command).await,
                    "docker" => slash::docker::run(&ctx, &command).await,
                    "garage" => slash::garage::run(&ctx, &command).await,
                    "meme" => slash::meme::run(&ctx, &command).await,
                    "missions" => slash::missions::run(&ctx, &command).await,
                    "reputation" => slash::reputation::run(&ctx, &command).await,
                    "reset" => slash::reset::run(&ctx, &command).await,
                    "schedule" => slash::schedule::run(&ctx, &command).await,
                    MENU_AAR_IDS => menu::missions::run_aar_ids(&ctx, &command).await,
                    MENU_AAR_PAY => menu::missions::run_aar_pay(&ctx, &command).await,
                    MENU_RECRUITING_REPLY => menu::recruiting::run_reply(&ctx, &command).await,
                    _ => Ok(()),
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                debug!(
                    "matching autocomplete: {:?}",
                    autocomplete.data.name.as_str()
                );
                match autocomplete.data.name.as_str() {
                    "certifications" => {
                        slash::certifications::autocomplete(&ctx, &autocomplete).await
                    }
                    "garage" => {
                        slash::garage::auto_complete::autocomplete(&ctx, &autocomplete).await
                    }
                    "docker" => slash::docker::autocomplete(&ctx, &autocomplete).await,
                    "missions" => slash::missions::autocomplete(&ctx, &autocomplete).await,
                    "reset" => slash::reset::autocomplete(&ctx, &autocomplete).await,
                    "schedule" => slash::schedule::autocomplete(&ctx, &autocomplete).await,
                    _ => Ok(()),
                }
            }
            Interaction::MessageComponent(component) => {
                debug!(
                    "matching component: {:?}",
                    component.data.custom_id.as_str()
                );
                match component.data.custom_id.as_str() {
                    "rsvp_yes" | "rsvp_maybe" | "rsvp_no" => {
                        slash::schedule::rsvp_button(&ctx, &component).await
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        } {
            error!("Cannot handle interaction: {}", e);
        }
    }

    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
        if new_member.user.bot {
            return;
        }
        if new_member.guild_id == synixe_meta::discord::GUILD {
            if let Err(e) = synixe_meta::discord::channel::LOBBY
                .send_message(&_ctx, |m| {
                    m.content(&format!(
                        "Welcome <@{}>! Please follow the steps in <#{}> to get prepared to jump in game with us. If you have any questions, feel free to ask here!",
                        new_member.user.id,
                        synixe_meta::discord::channel::ONBOARDING,
                    ))
                })
                .await {
                error!("Cannot send welcome message: {}", e);
            }
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        kicked: User,
        _member: Option<Member>,
    ) {
        if kicked.bot {
            return;
        }
        if guild_id == synixe_meta::discord::GUILD {
            if let Err(e) = synixe_meta::discord::channel::LOG
                .send_message(&ctx, |m| {
                    m.content(&format!(
                        "{}#{} ({}) has left, <@{}>",
                        kicked.name, kicked.discriminator, kicked.id, kicked.id
                    ))
                })
                .await
            {
                error!("Cannot send leave message: {}", e);
            }
        }
    }

    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<Member>,
        new: Member,
    ) {
        if new.user.bot {
            return;
        }
        if new.guild_id == synixe_meta::discord::GUILD {
            if let Err(e) = publish!(
                bootstrap::NC::get().await,
                Publish::MemberUpdate {
                    member: new.clone(),
                }
            )
            .await
            {
                error!("Cannot publish member update: {}", e);
            }
        }
        if new.roles.contains(&synixe_meta::discord::role::RECRUIT) {
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositSearch(Ok(deposits)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankDepositSearch {
                    member: new.user.id,
                    reason: Some("Starting Funds".to_string()),
                    id: None,
                }
            ).await else {
                error!("Cannot get starting funds for {}", new.user.id);
                return;
            };
            if !deposits.is_empty() {
                return;
            }
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositNew(Ok(())), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankDepositNew {
                    member: new.user.id,
                    reason: "Starting Funds".to_string(),
                    amount: 3500,
                    id: Some(Uuid::nil()),
                }
            ).await else {
                error!("Failed to create starting funds {}", new.user.id);
                return;
            };
        }
    }

    async fn guild_ban_addition(&self, ctx: Context, guild_id: GuildId, banned_user: User) {
        if banned_user.bot {
            return;
        }
        if guild_id == synixe_meta::discord::GUILD {
            if let Err(e) = synixe_meta::discord::channel::LOG
                .send_message(&ctx, |m| {
                    m.content(&format!(
                        "{}#{} ({}) was banned, <@{}>",
                        banned_user.name, banned_user.discriminator, banned_user.id, banned_user.id
                    ))
                })
                .await
            {
                error!("Cannot send ban message: {}", e);
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.channel_id == FINANCIALS {
            missions::validate_aar(&ctx, message).await;
            return;
        }

        if [ONTOPIC, OFFTOPIC, BOT, LOOKING_TO_PLAY, STAFF, LOBBY].contains(&message.channel_id) {
            if message.author.bot {
                return;
            }

            if let Some(brain) = &self.brain {
                let mut name = message.author.name.clone();
                if let Ok(author) = GUILD.member(&ctx, message.author.id).await {
                    if let Some(nick) = &author.nick {
                        name = nick.clone();
                    }
                }
                let mut content = format!("{}: {}", name, message.content);
                for user in &message.mentions {
                    if let Ok(member) = GUILD.member(&ctx, user.id).await {
                        if let Some(nick) = &member.nick {
                            content = content.replace(&format!("<@{}>", user.id), nick);
                        }
                    }
                }
                content = content.replace("<@1028418063168708638>", "Ctirad Brodsky");
                if message
                    .mentions
                    .iter()
                    .any(|user| user.id == ctx.cache.current_user_id())
                {
                    if let Some(reply) = brain.message(message.channel_id, content).await {
                        message.reply_ping(&ctx.http, reply).await.ok();
                    }
                } else if rand::thread_rng().gen_range(0..100) < 2 {
                    if let Some(reply) = brain.message(message.channel_id, content).await {
                        message.reply_ping(&ctx.http, reply).await.ok();
                    }
                } else {
                    brain.context(message.channel_id, content).await;
                }
            }
        }

        // 0.02% chance of losing 10 Harrison Points
        if rand::thread_rng().gen_range(0..10000) < 2 {
            if let Err(e) = message
                .reply(
                    &ctx.http,
                    "I can't believe you've said this! You just lost 10 Harrison Points!",
                )
                .await
            {
                error!("Cannot send message: {}", e);
            }
        }

        let lower = message.content.to_lowercase();
        for key in &[
            "more guns",
            "more weapons",
            "more gear",
            "more mods",
            "more equipment",
        ] {
            if lower.contains(key) {
                if let Err(e) = message
                    .reply(&ctx.http, "https://www.youtube.com/watch?v=H5d42w4ZcY4")
                    .await
                {
                    error!("Cannot send message: {}", e);
                }
            }
        }
    }

    #[allow(clippy::manual_let_else)]
    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if event.channel_id == FINANCIALS {
            #[allow(clippy::single_match_else)]
            let message = match new {
                Some(message) => message,
                None => {
                    if let Some(content) = event.content {
                        if !(content.starts_with("```") || content.ends_with("```")) {
                            return;
                        }
                    }
                    let Ok(message) = event.channel_id.message(&ctx.http, event.id).await else {
                        warn!("Cannot get message {}", event.id);
                        return;
                    };
                    message
                }
            };
            missions::validate_aar(&ctx, message).await;
        }
    }
}
