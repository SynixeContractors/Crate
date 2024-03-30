use std::sync::atomic::AtomicU32;

use rand::Rng;
use serenity::{async_trait, model::prelude::*, prelude::*};
use synixe_events::{discord::publish::Publish, missions::db::Response, publish};
use synixe_meta::discord::channel::{
    BOT, GAME_LOG, LEADERSHIP, LOBBY, LOOKING_TO_PLAY, OFFTOPIC, ONTOPIC,
};
use synixe_proc::{events_request_2, events_request_5};
use uuid::Uuid;

use crate::{
    bot::Bot,
    discord::menu::missions::{MENU_AAR_IDS, MENU_AAR_PAY},
};

use super::{
    menu,
    slash::{self, schedule::post_mission},
};

mod brain;
mod missions;
// pub mod recruiting;

pub use self::brain::Brain;

pub struct Handler {
    pub brain: Brain,
    pub subcon_counter: AtomicU32,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        Bot::init(ctx.shard.into());

        if let Err(e) = synixe_meta::discord::GUILD
            .set_commands(
                &ctx.http,
                vec![
                    menu::missions::aar_ids(),
                    menu::missions::aar_pay(),
                    slash::bank::register(),
                    slash::certifications::register(),
                    slash::docker::register(),
                    slash::garage::register(),
                    slash::gear::register(),
                    slash::missions::register(),
                    slash::reputation::register(),
                    slash::schedule::register(),
                    slash::surveys::register(),
                ],
            )
            .await
        {
            error!("Cannot register slash commands: {}", e);
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Err(e) = match interaction {
            Interaction::Command(command) => {
                debug!("matching command: {:?}", command.data.name.as_str());
                match command.data.name.as_str() {
                    "bank" => slash::bank::run(&ctx, &command).await,
                    "certifications" => slash::certifications::run(&ctx, &command).await,
                    "docker" => slash::docker::run(&ctx, &command).await,
                    "garage" => slash::garage::run(&ctx, &command).await,
                    "gear" => slash::gear::run(&ctx, &command).await,
                    "missions" => slash::missions::run(&ctx, &command).await,
                    "reputation" => slash::reputation::run(&ctx, &command).await,
                    "schedule" => slash::schedule::run(&ctx, &command).await,
                    "surveys" => slash::surveys::run(&ctx, &command).await,
                    MENU_AAR_IDS => menu::missions::run_aar_ids(&ctx, &command).await,
                    MENU_AAR_PAY => menu::missions::run_aar_pay(&ctx, &command).await,
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
                    "docker" => slash::docker::autocomplete(&ctx, &autocomplete).await,
                    "garage" => {
                        slash::garage::auto_complete::autocomplete(&ctx, &autocomplete).await
                    }
                    "gear" => slash::gear::autocomplete(&ctx, &autocomplete).await,
                    "missions" => slash::missions::autocomplete(&ctx, &autocomplete).await,
                    "schedule" => slash::schedule::autocomplete(&ctx, &autocomplete).await,
                    "surveys" => slash::surveys::autocomplete(&ctx, &autocomplete).await,
                    _ => Ok(()),
                }
            }
            Interaction::Component(component) => {
                debug!(
                    "matching component: {:?}",
                    component.data.custom_id.as_str()
                );
                if component.data.custom_id.starts_with("survey_submit:") {
                    if let Err(e) = slash::surveys::submit_button(&ctx, &component).await {
                        error!("Cannot handle survey submit: {}", e);
                    }
                }
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
                .say(
                    &_ctx,
                    &format!(
                        "Welcome <@{}>! Please follow the steps in <#{}> to get prepared to jump in game with us. If you have any questions, feel free to ask here or reply to this post, I may know the answer!",
                        new_member.user.id,
                        synixe_meta::discord::channel::ONBOARDING,
                    ),
                )
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
                .say(
                    &ctx,
                    &format!(
                        "{} ({}) has left, <@{}>",
                        if let Some(discriminator) = kicked.discriminator {
                            format!("{}#{}", kicked.name, discriminator)
                        } else {
                            kicked.name
                        },
                        kicked.id,
                        kicked.id
                    ),
                )
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
        _new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        if event.user.bot {
            return;
        }
        if event.guild_id == synixe_meta::discord::GUILD {
            if let Err(e) = publish!(
                bootstrap::NC::get().await,
                Publish::MemberUpdate {
                    member: event.clone()
                }
            )
            .await
            {
                error!("Cannot publish member update: {}", e);
            }
        }
        if event.roles.contains(&synixe_meta::discord::role::RECRUIT) {
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositSearch(Ok(deposits)), _))) =
                events_request_2!(
                    bootstrap::NC::get().await,
                    synixe_events::gear::db,
                    BankDepositSearch {
                        member: event.user.id,
                        reason: Some("Starting Funds".to_string()),
                        id: None,
                    }
                )
                .await
            else {
                error!("Cannot get starting funds for {}", event.user.id);
                return;
            };
            if !deposits.is_empty() {
                return;
            }
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositNew(Ok(())), _))) =
                events_request_2!(
                    bootstrap::NC::get().await,
                    synixe_events::gear::db,
                    BankDepositNew {
                        member: event.user.id,
                        reason: "Starting Funds".to_string(),
                        amount: 3500,
                        id: Some(Uuid::nil()),
                    }
                )
                .await
            else {
                error!("Failed to create starting funds {}", event.user.id);
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
                .say(
                    &ctx,
                    &format!(
                        "{} ({}) was banned, <@{}>",
                        if let Some(discriminator) = banned_user.discriminator {
                            format!("{}#{}", banned_user.name, discriminator)
                        } else {
                            banned_user.name
                        },
                        banned_user.id,
                        banned_user.id
                    ),
                )
                .await
            {
                error!("Cannot send ban message: {}", e);
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn message(&self, ctx: Context, message: Message) {
        if message.channel_id == LEADERSHIP {
            missions::validate_aar(&ctx, message).await;
            return;
        }

        // if message.channel_id == RECRUITING && message.author.bot {
        //     recruiting::check_embed(&ctx, &message).await;
        //     return;
        // }

        if message.content.as_str() == "!exec mission-bump" {
            if let Err(e) = message.delete(&ctx.http).await {
                error!("Cannot delete message: {}", e);
            };
            move_channel_missions(&ctx, message.channel_id).await;
            return;
        }

        if message.channel_id == BOT && message.content.as_str() == "!exec active" {
            if let Err(e) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::discord::executions,
                UpdateActivityRoles {}
            )
            .await
            {
                message
                    .reply_ping(&ctx, format!("Cannot update activity roles: {e}"))
                    .await
                    .expect("Cannot send message");
            }
            return;
        }

        if message.channel_id == LOOKING_TO_PLAY {
            if message.author.id == ctx.cache.current_user().id && !message.embeds.is_empty() {
                self.subcon_counter
                    .store(0, std::sync::atomic::Ordering::Relaxed);
            } else {
                let since = self
                    .subcon_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if (since + 1) % 15 == 0 {
                    move_channel_missions(&ctx, message.channel_id).await;
                }
            }
        }

        if [ONTOPIC, OFFTOPIC, BOT, LOOKING_TO_PLAY, LOBBY].contains(&message.channel_id) {
            if message.author.bot {
                return;
            }

            if self.brain.awake() {
                if message.content.is_empty() {
                    return;
                }
                if message
                    .mentions
                    .iter()
                    .any(|user| user.id == ctx.cache.current_user().id)
                {
                    let typing = message.channel_id.start_typing(&ctx.http);
                    if let Some(reply) = self.brain.ask(&ctx, &message).await {
                        match message.reply_ping(&ctx.http, reply).await {
                            Ok(reply) => self.brain.observe(&ctx, &reply).await,
                            Err(e) => error!("Cannot send message: {}", e),
                        }
                    }
                    typing.stop();
                } else if rand::thread_rng().gen_range(0..100) < 4 {
                    let typing = message.channel_id.start_typing(&ctx.http);
                    if let Some(reply) = self.brain.ask(&ctx, &message).await {
                        match message.reply_ping(&ctx.http, reply).await {
                            Ok(reply) => self.brain.observe(&ctx, &reply).await,
                            Err(e) => error!("Cannot send message: {}", e),
                        }
                    } else {
                        warn!("No reply could be generated");
                    }
                    typing.stop();
                } else {
                    self.brain.observe(&ctx, &message).await;
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
        if event.channel_id == LEADERSHIP {
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

    async fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        if channel_id == GAME_LOG {
            if let Err(e) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::reputation::db,
                DeleteByMessage {
                    message: message_id,
                }
            )
            .await
            {
                error!("Cannot delete reputation event: {}", e);
            }
        }
    }
}

async fn move_channel_missions(ctx: &Context, channel: ChannelId) {
    // delete subcon message and move here
    let Ok(Ok((Response::FetchUpcomingChannel(Ok(evs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchUpcomingChannel { channel }
    )
    .await
    else {
        error!("Cannot fetch scheduled message");
        return;
    };
    for mission in evs {
        let Some(message_id) = &mission.schedule_message_id else {
            continue;
        };
        let (_channel, message_id) = message_id.split_once(':').expect("Invalid message id");
        let Ok(subcon_message) = LOOKING_TO_PLAY
            .message(
                &ctx.http,
                MessageId::new(message_id.parse().expect("invalid message id")),
            )
            .await
        else {
            continue;
        };
        if let Err(e) = subcon_message.delete(&ctx.http).await {
            error!("Cannot delete subcon message: {}", e);
        }
        let Some(new_mission) = post_mission(ctx, channel, &mission, None, false).await else {
            error!("Cannot post mission");
            return;
        };
        if let Err(e) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            SetScheduledMesssage {
                channel: LOOKING_TO_PLAY,
                message: new_mission.id,
                scheduled: mission.id,
            }
        )
        .await
        {
            error!("Cannot set scheduled message: {}", e);
        }
    }
}
