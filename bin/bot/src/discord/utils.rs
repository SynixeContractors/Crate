use serenity::{model::prelude::UserId, prelude::Context};
use synixe_meta::discord::GUILD;

pub async fn find_members(ctx: &Context, names: &[&str]) -> Result<Vec<UserId>, String> {
    let Ok(members) = GUILD.members(&ctx.http, None, None).await else {
        return Err("Failed to fetch members".to_string())
    };
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        // Handle the special snowflake
        if name == &"Nathanial Greene" {
            ids.push(UserId(358_146_229_626_077_187));
            continue;
        }
        let Some(member) = members.iter().find(|m| m.display_name().to_string().to_lowercase() == name.to_lowercase()) else {
            return Err(format!("Failed to find member {name}"));
        };
        ids.push(member.user.id);
    }
    Ok(ids)
}
