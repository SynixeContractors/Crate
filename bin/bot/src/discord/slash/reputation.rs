use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::reputation;
use synixe_proc::events_request_2;
use time::OffsetDateTime;

use crate::discord::interaction::{Generic, Interaction};

use super::AllowPublic;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("reputation")
        .description("Manage reputation")
        .create_option(|option| {
            option
                .name("view")
                .description("View our current reputation")
                .kind(CommandOptionType::SubCommand)
                .allow_public()
        })
    // .create_option(|option| {
    //     option
    //         .name("event")
    //         .description("Add a reputation event")
    //         .kind(CommandOptionType::SubCommand)
    //         .create_sub_option(|option| {
    //             option
    //                 .name("member")
    //                 .description("The member that the event is for (use Brodsky if ambiguous)")
    //                 .kind(CommandOptionType::User)
    //                 .required(true)
    //         })
    //         .create_sub_option(|option| {
    //             option
    //                 .name("description")
    //                 .description("The description of the event")
    //                 .kind(CommandOptionType::String)
    //                 .required(true)
    //         })
    //         .create_sub_option(|option| {
    //             option
    //                 .name("significance")
    //                 .description("The significance of the event")
    //                 .kind(CommandOptionType::Integer)
    //                 .required(true)
    //                 .min_int_value(-200)
    //                 .max_int_value(200)
    //         })
    //     })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "view" => view(ctx, command, &subcommand.options).await?,
            // "event" => event(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    let Ok(Ok((reputation::db::Response::CurrentReputation(Ok(Some(Some(current_rep)))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reputation::db,
        CurrentReputation {
            at: OffsetDateTime::now_utc(),
        }
    )
    .await else {
        return interaction.reply("Failed to transfer money").await;
    };
    interaction
        .reply(format!("Current reputation: {current_rep}"))
        .await?;
    Ok(())
}

// async fn event(
//     ctx: &Context,
//     command: &ApplicationCommandInteraction,
//     options: &[CommandDataOption],
// ) -> serenity::Result<()> {
//     let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
//     let Some(member) = get_option_user!(options, "member") else {
//         return interaction.reply("Invalid trainee").await;
//     };
//     let description = get_option!(options, "description", String);
//     let significance = get_option!(options, "significance", Integer);
//     Ok(())
// }
