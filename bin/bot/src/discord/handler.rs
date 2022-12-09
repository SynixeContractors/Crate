use opentelemetry::{
    trace::{FutureExt, Span, TraceContextExt, Tracer},
    KeyValue,
};
use serenity::{async_trait, model::prelude::*, prelude::*};
use synixe_events::{discord::publish::Publish, publish};
use synixe_proc::events_request;
use uuid::Uuid;

use super::{menu, slash};

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
            })
            .await
        {
            error!("Cannot register slash commands: {}", e);
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                debug!("matching command: {:?}", command.data.name.as_str());
                let tracer = bootstrap::tracer!("bot");
                let mut span = tracer.start(format!("command:{}", command.data.name.as_str()));
                span.set_attribute(KeyValue::new(
                    "command.data".to_string(),
                    serde_json::to_string(&command.data).unwrap(),
                ));
                let cx = opentelemetry::Context::new().with_span(span);
                match command.data.name.as_str() {
                    "bank" => {
                        slash::bank::run(&ctx, &command).with_context(cx).await;
                    }
                    "certifications" => {
                        slash::certifications::run(&ctx, &command)
                            .with_context(cx)
                            .await;
                    }
                    "meme" => slash::meme::run(&ctx, &command).with_context(cx).await,
                    "schedule" => {
                        slash::missions::schedule_run(&ctx, &command)
                            .with_context(cx)
                            .await;
                    }
                    "Recruiting - Reply" => {
                        menu::recruiting::run_reply(&ctx, &command)
                            .with_context(cx)
                            .await;
                    }
                    _ => {}
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                debug!(
                    "matching autocomplete: {:?}",
                    autocomplete.data.name.as_str()
                );
                let tracer = bootstrap::tracer!("bot");
                let mut span =
                    tracer.start(format!("autocomplete:{}", autocomplete.data.name.as_str()));
                span.set_attribute(KeyValue::new(
                    "autocomplete.data".to_string(),
                    serde_json::to_string(&autocomplete.data).unwrap(),
                ));
                let cx = opentelemetry::Context::new().with_span(span);
                match autocomplete.data.name.as_str() {
                    "schedule" => {
                        slash::missions::schedule_autocomplete(&ctx, &autocomplete)
                            .with_context(cx)
                            .await;
                    }
                    "certifications" => {
                        slash::certifications::autocomplete(&ctx, &autocomplete)
                            .with_context(cx)
                            .await;
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
            let Ok(((synixe_events::gear::db::Response::BankDepositSearch(Ok(deposits)), _), _)) = events_request!(
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
            let Ok(((synixe_events::gear::db::Response::BankDepositNew(Ok(())), _), _)) = events_request!(
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

    async fn reaction_add(&self, _ctx: Context, reaction: Reaction) {
        publish!(
            bootstrap::NC::get().await,
            Publish::ReactionAdd { reaction }
        )
        .await
        .unwrap();
    }

    async fn reaction_remove(&self, _ctx: Context, reaction: Reaction) {
        publish!(
            bootstrap::NC::get().await,
            Publish::ReactionRemove { reaction }
        )
        .await
        .unwrap();
    }
}
