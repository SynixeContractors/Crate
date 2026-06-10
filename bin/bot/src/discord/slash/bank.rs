use charts_rs::{CandlestickChart, PieChart, THEME_GRAFANA, svg_to_png};
use serenity::{
    all::{
        CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType,
        CreateAttachment,
    },
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::gear::db::{Response, Transaction};
use synixe_meta::discord::{
    BRODSKY, GUILD,
    channel::LOG,
    role::{ACTIVE, STAFF},
};
use synixe_proc::events_request_5;
use time::format_description;

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
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "spent", "View money spent per category for a member")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view spending for")
                    .required(true)
                )
                .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "candlestick", "View a candlestick chart of a member's balance over time")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view the chart for")
                    .required(true)
                )
                .allow_public()
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
            "spent" => spent(ctx, command, options).await?,
            "candlestick" => candlestick(ctx, command, options).await?,
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

    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_5!(
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

    let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch locker balance").await;
    };
    let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_5!(
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

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
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

    if user == &BRODSKY {
        return interaction.reply("I don't accept bribes").await;
    }

    super::requires_roles(
        command.user.id,
        &[ACTIVE],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("bank transfer", options)),
        &mut interaction,
    )
    .await?;

    if user != &synixe_meta::discord::BRODSKY && GUILD.member(&ctx, user).await?.user.bot {
        return interaction.reply("You can't transfer money to a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankTransferNew(Ok(())), _))) = events_request_5!(
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

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
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
    let Ok(Ok((Response::BankDepositNew(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankDepositNew {
            member: *user,
            #[allow(clippy::cast_possible_truncation)]
            amount: -*amount as i32,
            reason: format!("fine from {}: {reason}", command.user.id),
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

    Ok(())
}

async fn spent(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching spending data...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };

    let Ok(Ok((Response::BankSpent(Ok(spending)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankSpent { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch spending data").await;
    };

    if spending.is_empty() {
        return interaction
            .reply("No spending data found for this member")
            .await;
    }

    let reply = format!("Spending for <@{user}>:\n");
    // sort spending by cost descending
    let mut spending = spending
        .into_iter()
        .filter(|(_, cost)| *cost != 0)
        .collect::<Vec<_>>();
    spending.sort_by_key(|b| std::cmp::Reverse(b.1));
    let mut chart = PieChart::new_with_theme(
        spending
            .iter()
            .map(|(category, cost)| (category.as_str(), vec![*cost as f32]).into())
            .collect(),
        THEME_GRAFANA,
    );
    chart.legend_show = Some(false);
    chart.series_label_formatter = String::from("{a}: ${c} ({d}%)");

    let image = svg_to_png(&chart.svg().expect("Failed to generate SVG"))
        .expect("Failed to convert SVG to PNG");
    interaction
        .reply_with_attachment(reply, CreateAttachment::bytes(image, "chart.png"))
        .await
}

async fn candlestick(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching balance history...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };

    let Ok(Ok((Response::BankHistory(Ok(history)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankHistory { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch balance history").await;
    };

    if history.is_empty() {
        return interaction
            .reply("No balance history found for this member")
            .await;
    }

    // Reverse history to get oldest to newest (it comes sorted DESC from the DB)
    let history: Vec<_> = history.into_iter().rev().collect();

    // group history by day, then find the open, high, low, and close for each day
    let mut current_balance = 0;
    let mut daily_balances: Vec<(String, i32, i32, i32, i32)> = Vec::new();
    let mut current_day = String::new();
    let mut open = 0;
    let mut high = i32::MIN;
    let mut low = i32::MAX;
    let format =
        format_description::parse("[year]-[month]-[day]").expect("Failed to parse date format");
    for Transaction { amount, created } in history {
        let day = created
            .format(&format)
            .expect("Failed to format date")
            .clone();
        if day != current_day {
            if !current_day.is_empty() {
                daily_balances.push((current_day.clone(), open, high, low, current_balance));
            }
            current_day = day;
            open = current_balance;
            high = current_balance;
            low = current_balance;
        }
        current_balance += amount;
        if current_balance > high {
            high = current_balance;
        }
        if current_balance < low {
            low = current_balance;
        }
    }
    // push the last day
    daily_balances.push((current_day, open, high, low, current_balance));

    // [open price1, close price1, lowest price1, highest price1, open price2, close price2, ...]
    let mut data = Vec::new();
    #[allow(clippy::cast_precision_loss)]
    for (_, open, high, low, close) in &daily_balances {
        data.push(*open as f32);
        data.push(*close as f32);
        data.push(*low as f32);
        data.push(*high as f32);
    }

    let x_axis_data: Vec<String> = daily_balances
        .iter()
        .map(|(day, _, _, _, _)| day.clone())
        .collect();

    let mut chart = CandlestickChart::new_with_theme(
        vec![("Balance", data).into()],
        x_axis_data,
        THEME_GRAFANA,
    );

    // Swap Asian colors (red for up, green for down) to Western colors (green for up, red for down)
    let up_color = chart.candlestick_down_color;
    let up_border_color = chart.candlestick_down_border_color;
    chart.candlestick_down_color = chart.candlestick_up_color;
    chart.candlestick_down_border_color = chart.candlestick_up_border_color;
    chart.candlestick_up_color = up_color;
    chart.candlestick_up_border_color = up_border_color;

    let image = svg_to_png(&chart.svg().expect("Failed to generate SVG"))
        .expect("Failed to convert SVG to PNG");
    interaction
        .reply_with_attachment(
            format!("Balance history for <@{user}>:"),
            CreateAttachment::bytes(image, "chart.png"),
        )
        .await
}
