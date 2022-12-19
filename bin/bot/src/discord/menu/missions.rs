use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandType, MessageId,
    },
    prelude::Context,
};
use synixe_meta::discord::GUILD;

use crate::discord::interaction::{Generic, Interaction};

pub fn aar_ids(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("AAR - Get IDs").kind(CommandType::Message)
}

pub async fn run_aar_ids(ctx: &Context, command: &ApplicationCommandInteraction) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    let Ok(msg) = command.channel_id
        .message(&ctx.http, MessageId::from(command.data.target_id.unwrap()))
        .await
    else {
        interaction
            .reply("Failed to find message")
            .await;
        return;
    };
    let Some(data) = msg.content.lines().into_iter().find(|l| l.starts_with("Contractors: ")) else {
        interaction
            .reply("Failed to find contractors list")
            .await;
        return;
    };
    let names = data
        .trim_start_matches("Contractors: ")
        .split(", ")
        .collect::<Vec<_>>();
    let Ok(members) = GUILD.members(&ctx.http, None, None).await else {
        interaction
            .reply("Failed to get members")
            .await;
        return;
    };
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        // Handle the special snowflake
        if name == "Nathanial Greene" {
            ids.push("358146229626077187".to_string());
            continue;
        }
        let Some(member) = members.iter().find(|m| m.display_name().to_string().to_lowercase() == name.to_lowercase()) else {
            interaction
                .reply(format!("Failed to find member {name}"))
                .await;
            return;
        };
        ids.push(member.user.id.0.to_string());
    }
    interaction
        .reply(format!("**IDs**\n{}", ids.join("\n")))
        .await;
}
