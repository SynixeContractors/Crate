use serde_json::json;
use serenity::{model::Timestamp, prelude::Context};
use synixe_meta::discord::{GUILD, channel::LOG, role::STAFF};
use time::{Duration, OffsetDateTime};

use crate::discord::utils::find_members;

use super::{BrainFunction, Caller};

/// The longest timeout Discord allows, in minutes
const MAX_MINUTES: f64 = 28.0 * 24.0 * 60.0;
/// The longest timeout Brodsky gives on his own, when someone who isn't staff is abusing him
const SELF_DEFENCE_MINUTES: f64 = 10.0;

pub struct Timeout {}

#[async_trait::async_trait]
impl BrainFunction for Timeout {
    fn name(&self) -> &'static str {
        "timeout"
    }

    fn desc(&self) -> &'static str {
        "Timeout a user for being genuinely abusive or breaking the rules. Anyone abusing you can be timed out, for at most 10 minutes. Only staff can ask you to time out someone else."
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "names": {
                    "type": "array",
                    "description": "name of the users to impose timeout on",
                    "items": {
                        "type": "string",
                    },
                },
                "duration": {
                    "type": "number",
                    "description": "duration of the timeout in minutes, under 5 for minor offenses (innapropriate behaviour, joke too far), 20+ for major offenses, 60+ for severe offenses (hate speech)",
                },
                "reason": {
                    "type": "string",
                    "description": "reason for the timeout",
                },
            }
        })
    }

    async fn run(
        &self,
        ctx: &Context,
        caller: &Caller,
        args: serde_json::Value,
    ) -> Option<serde_json::Value> {
        info!("running moderation function for {}: {:?}", caller.id, args);
        let names = args["names"]
            .as_array()?
            .iter()
            // The model can send anything, so names that aren't a string or ID are skipped
            .filter_map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .or_else(|| v.as_u64().map(|v| v.to_string()))
            })
            .collect::<Vec<_>>();
        // Declared as a number, so it may arrive as a float
        let limit = if caller.staff {
            MAX_MINUTES
        } else {
            SELF_DEFENCE_MINUTES
        };
        #[allow(clippy::cast_possible_truncation)]
        let minutes = args["duration"].as_f64()?.round().clamp(1.0, limit) as i64;
        let (found, _) = match find_members(ctx, &names).await {
            Ok(members) => members,
            Err(e) => {
                error!("failed to find members: {}", e);
                return Some(json!({
                    "error": e,
                }));
            }
        };
        // Without staff asking, Brodsky can only time out the person talking to him, so he can't be
        // talked into timing out someone else
        if !caller.staff && found.iter().any(|(_, id)| *id != caller.id) {
            return Some(json!({
                "error": "only staff can ask you to time out someone else",
            }));
        }
        for (_, id) in found {
            let Ok(mut member) = GUILD.member(&ctx, id).await else {
                error!("failed to get member: {}", id);
                continue;
            };
            if !caller.staff && member.roles.contains(&STAFF) {
                return Some(json!({
                    "error": "you can't time out staff",
                }));
            }
            if let Some(until) = member.communication_disabled_until
                && until > Timestamp::now()
            {
                info!("user already timed out: {}", member.user.id);
                continue;
            }
            info!("timing out user: {}", member.user.id);
            if let Err(e) = member
                .disable_communication_until_datetime(
                    ctx,
                    (OffsetDateTime::now_utc() + Duration::minutes(minutes)).into(),
                )
                .await
            {
                error!("failed to timeout user: {}", e);
                return Some(json!({
                    "error": format!("failed to timeout user: {}", e),
                }));
            }
            if let Err(e) = LOG
                .say(
                    ctx,
                    format!(
                        "Timed out user <@{}> for {minutes} minutes\nReason: {}",
                        member.user.id,
                        args["reason"].as_str().unwrap_or("No reason provided")
                    ),
                )
                .await
            {
                error!("failed to log timeout: {}", e);
            }
            let Ok(private_channel) = member.user.create_dm_channel(&ctx).await else {
                error!(
                    "Unable to create DM channel for user timeout: {}",
                    member.user.id
                );
                continue;
            };
            private_channel
                .say(
                    &ctx,
                    format!(
                        "You have been timed out for {minutes} minutes\nReason: {}",
                        args["reason"].as_str().unwrap_or("No reason provided")
                    ),
                )
                .await
                .ok();
        }
        Some(json!({
            "success": "timed out users",
        }))
    }
}
