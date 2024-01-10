use serde_json::json;
use serenity::prelude::Context;
use synixe_meta::discord::{channel::LOG, GUILD};
use time::{Duration, OffsetDateTime};

use crate::discord::utils::find_members;

use super::BrainFunction;

pub struct Timeout {}

#[async_trait::async_trait]
impl BrainFunction for Timeout {
    fn name(&self) -> &'static str {
        "timeout"
    }

    fn desc(&self) -> &'static str {
        "Timeout a user for being innapropriate or breaking the rules"
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

    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        info!("running moderation function: {:?}", args);
        let names = args["names"]
            .as_array()?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .or_else(|| v.as_u64().map(|v| v.to_string()))
                    .expect("name is string or u64")
            })
            .collect::<Vec<_>>();
        let (found, _) = match find_members(ctx, &names).await {
            Ok(members) => members,
            Err(e) => {
                error!("failed to find members: {}", e);
                return Some(json!({
                    "error": e,
                }));
            }
        };
        for (_, id) in found {
            let Ok(mut member) = GUILD.member(&ctx, id).await else {
                error!("failed to get member: {}", id);
                continue;
            };
            if member.communication_disabled_until.is_some() {
                info!("user already timed out: {}", member.user.id);
                continue;
            }
            info!("timing out user: {}", member.user.id);
            if let Err(e) = member
                .disable_communication_until_datetime(
                    ctx,
                    (OffsetDateTime::now_utc() + Duration::minutes(args["duration"].as_i64()?))
                        .into(),
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
                        "Timed out user <@{}> for {} minutes\nReason: {}",
                        member.user.id,
                        args["duration"],
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
                        "You have been timed out for {} minutes\nReason: {}",
                        args["duration"], args["reason"]
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
