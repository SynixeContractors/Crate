use serenity::{async_trait, model::prelude::*, prelude::*};
use synixe_events::{discord::publish::Publish, publish};
use synixe_meta::discord::channel::FINANCIALS;
use synixe_proc::events_request;
use uuid::Uuid;

use super::{menu, slash};

mod missions;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        if let Err(e) =
            GuildId::set_application_commands(&synixe_meta::discord::GUILD, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| slash::bank::register(command))
                    .create_application_command(|command| slash::certifications::register(command))
                    .create_application_command(|command| slash::meme::register(command))
                    .create_application_command(|command| slash::missions::schedule(command))
                    .create_application_command(|command| menu::recruiting::reply(command))
                    .create_application_command(|command| menu::missions::aar_ids(command))
            })
            .await
        {
            error!("Cannot register slash commands: {}", e);
        }

        // ChannelId(833_129_840_193_699_860).message(&ctx.http, 1_053_912_971_597_848_726).await.unwrap().reply(&ctx.http, "I am unable to find the contractor 'Matias Jackson'. Please edit your AAR to include the correct name.").await.unwrap();
        // ChannelId(833_129_840_193_699_860).message(&ctx.http, 1_053_850_477_101_588_480).await.unwrap().reply(&ctx.http, "I am unable to find the contractor 'Sean Miles. Andrew Libby'. Please edit your AAR to include the correct name.").await.unwrap();
    }

    #[allow(clippy::too_many_lines)]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                debug!("matching command: {:?}", command.data.name.as_str());
                match command.data.name.as_str() {
                    "bank" => {
                        slash::bank::run(&ctx, &command).await;
                    }
                    "certifications" => {
                        slash::certifications::run(&ctx, &command).await;
                    }
                    "meme" => slash::meme::run(&ctx, &command).await,
                    "schedule" => {
                        slash::missions::schedule_run(&ctx, &command).await;
                    }
                    "Recruiting - Reply" => {
                        menu::recruiting::run_reply(&ctx, &command).await;
                    }
                    "AAR - Get IDs" => {
                        menu::missions::run_aar_ids(&ctx, &command).await;
                    }
                    _ => {}
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                debug!(
                    "matching autocomplete: {:?}",
                    autocomplete.data.name.as_str()
                );
                match autocomplete.data.name.as_str() {
                    "schedule" => {
                        slash::missions::schedule_autocomplete(&ctx, &autocomplete).await;
                    }
                    "certifications" => {
                        slash::certifications::autocomplete(&ctx, &autocomplete).await;
                    }
                    _ => {}
                }
            }
            Interaction::MessageComponent(component) => {
                debug!(
                    "matching component: {:?}",
                    component.data.custom_id.as_str()
                );
                match component.data.custom_id.as_str() {
                    "rsvp_yes" | "rsvp_maybe" | "rsvp_no" => {
                        slash::missions::rsvp_button(&ctx, &component).await;
                    }
                    _ => {}
                }
            }
            _ => (),
        }
    }

    async fn guild_member_addition(&self, _ctx: Context, new_member: Member) {
        if new_member.user.bot {
            return;
        }
        if new_member.guild_id == synixe_meta::discord::GUILD {
            synixe_meta::discord::channel::LOBBY
                .send_message(&_ctx, |m| {
                    m.content(&format!(
                        "Welcome <@{}>! Please follow the steps in <#{}> to get prepared to jump in game with us. If you have any questions, feel free to ask here!",
                        new_member.user.id,
                        synixe_meta::discord::channel::ONBOARDING,
                    ))
                })
                .await
                .unwrap();
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
            synixe_meta::discord::channel::LOG
                .send_message(&ctx, |m| {
                    m.content(&format!(
                        "{}#{} ({}) has left, <@{}>",
                        kicked.name, kicked.discriminator, kicked.id, kicked.id
                    ))
                })
                .await
                .unwrap();
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
            publish!(
                bootstrap::NC::get().await,
                Publish::MemberUpdate {
                    member: new.clone(),
                }
            )
            .await
            .unwrap();
        }
        if new.roles.contains(&synixe_meta::discord::role::RECRUIT) {
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositSearch(Ok(deposits)), _))) = events_request!(
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
            let Ok(Ok((synixe_events::gear::db::Response::BankDepositNew(Ok(())), _))) = events_request!(
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
            synixe_meta::discord::channel::LOG
                .send_message(&ctx, |m| {
                    m.content(&format!(
                        "{}#{} ({}) was banned, <@{}>",
                        banned_user.name, banned_user.discriminator, banned_user.id, banned_user.id
                    ))
                })
                .await
                .unwrap();
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.channel_id == FINANCIALS {
            missions::validate_aar(&ctx, message).await;
        }
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if event.channel_id == FINANCIALS {
            let message = match new {
                Some(message) => message,
                None => event.channel_id.message(&ctx.http, event.id).await.unwrap(),
            };
            missions::validate_aar(&ctx, message).await;
        }
    }
}