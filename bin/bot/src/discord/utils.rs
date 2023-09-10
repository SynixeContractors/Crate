use serenity::{model::prelude::UserId, prelude::Context};
use synixe_meta::discord::GUILD;

pub async fn find_members(
    ctx: &Context,
    names: &[String],
) -> Result<(Vec<(String, UserId)>, Vec<String>), String> {
    let Ok(members) = GUILD.members(&ctx.http, None, None).await else {
        return Err("Failed to fetch members".to_string());
    };
    let mut ids = Vec::with_capacity(names.len());
    let mut unknown = Vec::new();
    for name in names {
        let name = name.trim();
        // Handle the special snowflake
        if name == "Nathanial Greene" {
            ids.push((name.to_string(), UserId(358_146_229_626_077_187)));
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
