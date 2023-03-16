use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::reputation;
use synixe_meta::discord::role::STAFF;
use synixe_proc::events_request_2;
use time::OffsetDateTime;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option, get_option_user,
};

use super::{AllowPublic, ShouldAsk};

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
        .create_option(|option| {
            option
                .name("add")
                .description("Add a reputation event")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member that the event is for (use Brodsky if ambiguous)")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("amount")
                        .description("The amount of reputation to add")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("reason")
                        .description("The reason for the reputation event")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("remove")
                .description("Remove a reputation event")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member that the event is for (use Brodsky if ambiguous)")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("amount")
                        .description("The amount of reputation to remove")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("reason")
                        .description("The reason for the reputation event")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
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
            "add" => update(ctx, command, &subcommand.options, true).await?,
            "remove" => update(ctx, command, &subcommand.options, false).await?,
            // "event" => event(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn update(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
    add: bool,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    super::requires_roles(
        command.user.id,
        &[STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;
    let Some(member) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid staff id").await;
    };
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };

    #[allow(clippy::cast_possible_truncation)]
    let mut real_amount = *amount as i32;
    if !add {
        real_amount = -real_amount;
    }

    let Ok(Ok((reputation::db::Response::UpdateReputation(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reputation::db,
        UpdateReputation {
            member: member.id,
            reputation: real_amount,
            reason: reason.to_string(),
        }
    )
    .await else {
        return interaction.reply("Failed to update reputation").await;
    };
    interaction
        .reply(format!("Updated {real_amount} reputation"))
        .await?;
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
        return interaction.reply("Failed to retrieve current reputation").await;
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
