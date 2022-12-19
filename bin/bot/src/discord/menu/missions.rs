use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandType, MessageId,
    },
    prelude::Context,
};

use crate::discord::{
    interaction::{Generic, Interaction},
    utils::find_members,
};

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
    match find_members(ctx, &names).await {
        Ok(ids) => {
            let ids = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
            interaction
                .reply(format!("**IDs**\n{}", ids.join("\n")))
                .await;
        }
        Err(e) => {
            interaction
                .reply(format!("Failed to find members: {e}"))
                .await;
        }
    }
}
