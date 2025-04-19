use std::sync::Arc;

use serenity::{
    futures::{StreamExt, stream::iter},
    model::prelude::UserId,
    prelude::Context,
};
use synixe_events::discord::write::{DiscordContent, DiscordMessage};
use synixe_meta::discord::GUILD;
use synixe_proc::events_request_2;
use tokio::sync::Mutex;

/// Finds users by nicknames or user IDs
pub async fn find_members(
    ctx: &Context,
    names_or_ids: &[String],
) -> Result<(Vec<(String, UserId)>, Vec<String>), String> {
    let Ok(members) = GUILD.members(&ctx.http, None, None).await else {
        return Err("Failed to fetch members".to_string());
    };
    let ids = Arc::new(Mutex::new(Vec::with_capacity(names_or_ids.len())));
    let names = iter(names_or_ids)
        .filter(|v| async {
            if let Ok(id) = v.parse::<u64>() {
                if let Ok(member) = GUILD.member(&ctx.http, id).await {
                    ids.lock()
                        .await
                        .push((member.display_name().to_string(), UserId::new(id)));
                    false
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .await;
    let mut unknown = Vec::new();
    let mut ids = Arc::try_unwrap(ids)
        .expect("failed to unwrap Arc")
        .into_inner();
    for name in names {
        let name = name.trim();
        // Handle the special snowflake
        if name == "Nathanial Greene" {
            ids.push((name.to_string(), UserId::new(358_146_229_626_077_187)));
            continue;
        }
        members
            .iter()
            .find(|m| m.display_name().to_string().to_lowercase() == name.to_lowercase())
            .map_or_else(
                || unknown.push(name.to_string()),
                |member| ids.push((member.display_name().to_string(), member.user.id)),
            );
    }
    Ok((ids, unknown))
}

pub fn audit(message: String) {
    tokio::spawn(async {
        if let Err(e) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::discord::write,
            Audit {
                message: DiscordMessage {
                    content: DiscordContent::Text(message),
                    reactions: vec![],
                }
            }
        )
        .await
        {
            error!("Failed to audit: {}", e);
        }
    });
}
