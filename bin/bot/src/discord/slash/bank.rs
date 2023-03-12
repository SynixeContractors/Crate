use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        command::CommandOptionType,
        UserId,
    },
    prelude::Context,
};
use synixe_events::gear::db::Response;
use synixe_meta::discord::role::STAFF;
use synixe_proc::events_request_2;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option, get_option_user,
};

use super::{AllowPublic, ShouldAsk};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bank")
        .description("Interact with the bank")
        .create_option(|option| {
            option
                .name("balance")
                .description("View a member's balance")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to view the balance of (select Brodsky to view the company account)")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .allow_public()
        })
        .create_option(|option| {
            option
                .name("transfer")
                .description("Transfer money to another member")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to transfer money to")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("amount")
                        .description("The amount of money to transfer")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(10_000)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("reason")
                        .description("The reason for the transfer")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("fine")
                .description("Fine a member")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to fine")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("amount")
                        .description("The amount of money to fine")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(10_000)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("reason")
                        .description("The reason for the fine")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "balance" => balance(ctx, command, &subcommand.options).await?,
            "transfer" => transfer(ctx, command, &subcommand.options).await?,
            "fine" => fine(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn balance(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Fetching balance...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    let user_id = if user.id == synixe_meta::discord::BRODSKY {
        UserId::from(0)
    } else {
        user.id
    };

    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: user_id,
        }
    )
    .await else {
        return interaction.reply("Failed to fetch balance").await;
    };

    if user_id == 0 {
        return interaction
            .reply(format!(
                "<@{}> has:\n```Cash: {}\n```",
                synixe_meta::discord::BRODSKY,
                bootstrap::format::money(balance, false),
            ))
            .await;
    }

    let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance {
            member: user_id,
        }
    )
    .await else {
        return interaction.reply("Failed to fetch locker balance").await;
    };
    let  Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LoadoutBalance {
            member: user_id,
        }
    )
    .await else {
        return interaction.reply("Failed to fetch loudout balance").await;
    };
    interaction
        .reply(format!(
            "<@{}> has:\n```Cash:      {}\nLocker:    {}\nLoadout:   {}\nNet Worth: {}```",
            user_id,
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
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Transferring money...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user.bot && user.id.0 != 1_028_418_063_168_708_638 {
        return interaction.reply("You can't transfer money to a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankTransferNew(Ok(_)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source: command.member.as_ref().expect("member should always exist on guild commands").user.id,
            target: user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            reason: reason.clone(),
        }
    )
    .await else {
        return interaction.reply("Failed to transfer money").await;
    };
    let reply = format!(
        "Transferred {} to <@{}>",
        bootstrap::format::money(*amount as i32, false),
        user.id
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for transfer notification");
        return interaction.reply(&format!("{reply}, but I wasn't able to notify them")).await;
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
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
async fn fine(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
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

    interaction.reply("Fining...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user.bot {
        return interaction.reply("You can't fine a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankDepositNew(Ok(_)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankDepositNew {
            member: command.member.as_ref().expect("member should always exist on guild commands").user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount:  -*amount as i32,
            reason: reason.clone(),
            id: None,
        }
    )
    .await else {
        return interaction.reply("Failed to fine").await;
    };
    let reply = format!(
        "<Deliverd a fine of @{}> to {}",
        user.id,
        bootstrap::format::money(*amount as i32, false),
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for fine notification");
        return interaction.reply(&format!("{reply}, but I wasn't able to notify them")).await;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "<You were fined {}\n> {}",
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
    Ok(())
}
