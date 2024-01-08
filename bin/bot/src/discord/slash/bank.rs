use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::gear::db::Response;
use synixe_meta::discord::{channel::LOG, role::STAFF, BRODSKY, GUILD};
use synixe_proc::events_request_2;

use crate::{discord::interaction::Interaction, get_option, get_option_user};

use super::{AllowPublic, ShouldAsk};

pub fn register() -> CreateCommand {
    CreateCommand::new("bank")
        .description("Interact with the bank")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "balance", "View a member's balance")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view the balance of (select Brodsky to view the company account)")
                .required(true)
            )
            .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "transfer", "Transfer money to another member")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to transfer money to")
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of money to transfer")
                    .min_int_value(1)
                    .max_int_value(10_000)
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "The reason for the transfer")
                    .required(true)
                )
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "fine", "Fine a member")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to fine")
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of money to fine")
                    .min_int_value(1)
                    .max_int_value(10_000)
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "The reason for the fine")
                    .required(true)
                )
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "balance" => balance(ctx, command, options).await?,
            "transfer" => transfer(ctx, command, options).await?,
            "fine" => fine(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn balance(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching balance...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };

    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch balance").await;
    };

    if user == &BRODSKY {
        return interaction
            .reply(format!(
                "<@{}> has:\n```Cash: {}\n```",
                BRODSKY,
                bootstrap::format::money(balance, false),
            ))
            .await;
    }

    let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch locker balance").await;
    };
    let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LoadoutBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch loudout balance").await;
    };
    interaction
        .reply(format!(
            "<@{}> has:\n```Cash:      {}\nLocker:    {}\nLoadout:   {}\nNet Worth: {}```",
            user,
            bootstrap::format::money(balance, false),
            bootstrap::format::money(locker_balance, false),
            bootstrap::format::money(loadout_balance, false),
            bootstrap::format::money(balance + locker_balance + loadout_balance, false)
        ))
        .await
}

#[allow(clippy::cast_possible_truncation)]
async fn transfer(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Transferring money...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user != &synixe_meta::discord::BRODSKY && GUILD.member(&ctx, user).await?.user.bot {
        return interaction.reply("You can't transfer money to a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankTransferNew(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id,
            target: *user,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            reason: reason.clone(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to transfer money").await;
    };
    let reply = format!(
        "Transferred {} to <@{}>",
        bootstrap::format::money(*amount as i32, false),
        user
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for transfer notification");
        return interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "<@{}> transferred you {}\n> {}",
                command
                    .member
                    .as_ref()
                    .expect("member should always exist on guild commands")
                    .user
                    .id,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send dm: {}", e);
        interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await?;
    }

    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!(
                "<@{}> transferred <@{}> {}\n> {}",
                command
                    .member
                    .as_ref()
                    .expect("member should always exist on guild commands")
                    .user
                    .id,
                user,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send log: {}", e);
    }

    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
async fn fine(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
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

    interaction.reply("Fining...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user != &synixe_meta::discord::BRODSKY && GUILD.member(&ctx, user).await?.user.bot {
        return interaction.reply("You can't fine a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankDepositNew(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankDepositNew {
            member: *user,
            #[allow(clippy::cast_possible_truncation)]
            amount: -*amount as i32,
            reason: format!("{}: {reason}", command.user.id),
            id: None,
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    let reply = format!(
        "Delivered a fine of {} to <@{}>.",
        bootstrap::format::money(*amount as i32, false),
        user,
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for fine notification");
        return interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "You were fined {}\n> {}",
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send dm: {}", e);
        interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await?;
    }

    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!(
                "<@{}> fined <@{}> {}\n> {}",
                command.user.id,
                user,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send log: {}", e);
    }
    Ok(())
}
