use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::reputation;
use synixe_meta::discord::role::STAFF;
use synixe_proc::events_request_2;
use time::OffsetDateTime;

use crate::{discord::interaction::Interaction, get_option, get_option_user};

use super::{AllowPublic, ShouldAsk};

pub fn register() -> CreateCommand {
    CreateCommand::new("reputation")
        .description("Manage reputation")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "view",
                "View our current reputation",
            )
            .allow_public(),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "add",
                "Add reputation amount",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "member",
                    "The member that the event is for (use Brodsky if ambiguous)",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "amount",
                    "The amount of reputation to add",
                )
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The reason for the reputation event",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "subtract",
                "Subtract reputation amount",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "member",
                    "The member that the event is for (use Brodsky if ambiguous)",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "amount",
                    "The amount of reputation to subtract",
                )
                .min_int_value(1)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "The reason for the reputation event",
                )
                .required(true),
            ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "view" => view(ctx, command, options).await?,
            "add" => update(ctx, command, options, true).await?,
            "subtract" => update(ctx, command, options, false).await?,
            // "event" => event(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn update(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
    add: bool,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
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
            staff: command.user.id,
            member: *member,
            reputation: real_amount,
            reason: reason.to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to update reputation").await;
    };
    interaction
        .reply(format!("Updated {real_amount} reputation"))
        .await?;
    Ok(())
}

async fn view(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Ok(Ok((reputation::db::Response::CurrentReputation(Ok(Some(Some(current_rep)))), _))) =
        events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            CurrentReputation {
                at: OffsetDateTime::now_utc(),
            }
        )
        .await
    else {
        return interaction
            .reply("Failed to retrieve current reputation")
            .await;
    };
    interaction
        .reply(format!("Current reputation: {current_rep:.2}"))
        .await?;
    Ok(())
}
