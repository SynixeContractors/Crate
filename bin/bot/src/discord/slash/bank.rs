use charts_rs::{CandlestickChart, PieChart, THEME_GRAFANA, svg_to_png};
use serenity::{
    all::{
        ButtonStyle, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType, ComponentInteraction, CreateAttachment, UserId,
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
        EditInteractionResponse,
    },
    client::Context,
};
use synixe_events::gear::db::{BankRequest, BankRequestStatus, Response, Transaction};
use synixe_meta::discord::{
    BRODSKY, GUILD,
    channel::LOG,
    role::{ACTIVE, STAFF},
};
use synixe_proc::events_request_5;
use time::format_description;

use crate::{
    discord::{handler::brain::voice, interaction::Interaction, utils::dm},
    get_option, get_option_user,
};

use super::{
    AllowPublic, ShouldAsk,
    bank_pages::{self, Kind},
};

/// The most that can be transferred, or requested, at once
const MAX_TRANSFER: i32 = 10_000;

pub fn register() -> CreateCommand {
    CreateCommand::new("bank")
        .description("Interact with the bank")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "balance", "View a member's balance")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view the balance of, yourself if not given (select Brodsky for the company account)")
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
                    .max_int_value(u64::from(MAX_TRANSFER.unsigned_abs()))
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "The reason for the transfer")
                    .required(true)
                )
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "request", "Ask another member to send you money, e.g. to be paid back")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to request money from")
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of money to request")
                    .min_int_value(1)
                    .max_int_value(u64::from(MAX_TRANSFER.unsigned_abs()))
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "What the money is for")
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
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view spending for, yourself if not given")
                )
                .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "history", "View a member's most recent bank transactions")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view transactions for, yourself if not given")
                )
                .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "transfers", "View a member's most recent transfers, sent and received")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view transfers for, yourself if not given"))
                .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "requests", "View bank requests you've sent or been asked to pay")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view requests for, yourself if not given"))
                .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "candlestick", "View a candlestick chart of a member's balance over time")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view the chart for, yourself if not given")
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
            "request" => request(ctx, command, options).await?,
            "fine" => fine(ctx, command, options).await?,
            "spent" => spent(ctx, command, options).await?,
            "candlestick" => candlestick(ctx, command, options).await?,
            "history" => history(ctx, command, options).await?,
            "transfers" => transfers(ctx, command, options).await?,
            "requests" => requests(ctx, command, options).await?,
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
    let user = &get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);

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
    #[allow(clippy::cast_possible_truncation)]
    let amount = *amount as i32;
    let Some(notified) = send_money(ctx, command.user.id, *user, amount, reason).await else {
        return interaction.reply("Failed to transfer money").await;
    };
    let reply = format!(
        "Transferred {} to <@{user}>",
        bootstrap::format::money(amount, false)
    );
    if notified {
        interaction.reply(reply).await
    } else {
        interaction
            .reply(format!("{reply}, but I wasn't able to notify them"))
            .await
    }
}

/// Moves money from `source` to `target`, logs it, sends `source` a receipt and tells `target`, e.g. for
/// `/bank transfer` or a paid `/bank request`. `None` if the transfer failed, otherwise whether `target`
/// was told.
pub async fn send_money(
    ctx: &Context,
    source: UserId,
    target: UserId,
    amount: i32,
    reason: &str,
) -> Option<bool> {
    let Ok(Ok((Response::BankTransferNew(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source,
            target,
            amount,
            reason: reason.to_string(),
        }
    )
    .await
    else {
        return None;
    };
    let money = bootstrap::format::money(amount, false);

    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!("<@{source}> transferred <@{target}> {money}\n> {reason}"),
        )
        .await
    {
        error!("failed to send log: {}", e);
    }

    // A receipt for the sender, so there's a record of what they sent, e.g. when paying someone back
    let balance = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance { member: source }
    )
    .await
    {
        Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) => format!(
            "\nYour balance is now {}",
            bootstrap::format::money(balance, false)
        ),
        _ => String::new(),
    };
    dm(
        ctx,
        source,
        format!("You transferred {money} to <@{target}>\n> {reason}{balance}"),
    )
    .await;

    Some(
        dm(
            ctx,
            target,
            format!("<@{source}> transferred you {money}\n> {reason}"),
        )
        .await,
    )
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
    let remark = voice::react(&format!(
        "A contractor was just fined {} by staff, for: {reason}",
        bootstrap::format::money(*amount as i32, false),
    ))
    .await
    .map_or_else(String::new, |line| format!("\n> {line}"));
    interaction.reply(format!("{reply}{remark}")).await?;

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
    let user = &get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);

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
    // Chart values only, so the precision lost past 2^24 doesn't matter
    #[allow(clippy::cast_precision_loss)]
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
    let user = &get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);

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
    let format = format_description::parse_borrowed::<3>("[year]-[month]-[day]")
        .expect("Failed to parse date format");
    for Transaction {
        amount, created, ..
    } in history
    {
        let day = created.format(&format).expect("Failed to format date");
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

async fn history(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    list(ctx, command, options, Kind::History).await
}

async fn transfers(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    list(ctx, command, options, Kind::Transfers).await
}

async fn requests(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let member = get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);
    // Someone else's requests are only for staff
    if member != command.user.id {
        let is_staff = command
            .member
            .as_ref()
            .is_some_and(|m| m.roles.contains(&STAFF));
        if !is_staff {
            return interaction
                .reply("Only staff can see other members' requests")
                .await;
        }
    }
    interaction.reply("Fetching requests...").await?;
    send_first_page(&mut interaction, Kind::Requests, member).await
}

/// Shows the first page of a list, with its paging buttons
async fn list(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
    kind: Kind,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching...").await?;
    let member = get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);
    send_first_page(&mut interaction, kind, member).await
}

async fn send_first_page(
    interaction: &mut Interaction<'_>,
    kind: Kind,
    member: UserId,
) -> serenity::Result<()> {
    match bank_pages::first_page(kind, member).await {
        Some((content, components)) => interaction.reply_with_components(content, components).await,
        None => {
            interaction
                .reply(format!("Failed to fetch the {}", kind.title()))
                .await
        }
    }
}

/// Prefixes of the Pay and Decline buttons on a `/bank request`, followed by the request's ID
pub const REQUEST_PAY: &str = "bank_request_pay:";
pub const REQUEST_DECLINE: &str = "bank_request_decline:";

#[allow(clippy::cast_possible_truncation)]
async fn request(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Sending request...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user == &BRODSKY {
        return interaction.reply("I don't give out loans").await;
    }
    if user == &command.user.id {
        return interaction
            .reply("You can't request money from yourself")
            .await;
    }
    if GUILD.member(&ctx, user).await?.user.bot {
        return interaction
            .reply("You can't request money from a bot")
            .await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    // Bounded to 1..=MAX_TRANSFER by the command definition
    let amount = *amount as i32;
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let money = bootstrap::format::money(amount, false);
    let requester = command.user.id;

    let Ok(Ok((Response::CreateBankRequest(Ok(id)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        CreateBankRequest {
            requester,
            payer: *user,
            amount,
            reason: reason.clone(),
        }
    )
    .await
    else {
        return interaction
            .reply("Failed to save your request, try again")
            .await;
    };

    let sent = match user.create_dm_channel(ctx).await {
        Ok(channel) => channel
            .send_message(
                &ctx.http,
                CreateMessage::new()
                    .content(format!(
                        "<@{requester}> is requesting {money} from you\n> {reason}"
                    ))
                    .components(vec![request_buttons(id, &money)]),
            )
            .await
            .map_err(|e| error!("failed to send bank request to {user}: {e}"))
            .is_ok(),
        Err(e) => {
            error!("unable to create dm channel for {user}: {e}");
            false
        }
    };
    if sent {
        interaction
            .reply(format!(
                "Requested {money} from <@{user}>. They've been sent a message to pay or decline. Track it with `/bank requests`."
            ))
            .await
    } else {
        interaction
            .reply(format!(
                "I wasn't able to message <@{user}>, they may have direct messages turned off. The request is saved, so it'll show in `/bank requests`."
            ))
            .await
    }
}

fn request_buttons(id: uuid::Uuid, money: &str) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{REQUEST_PAY}{id}"))
            .style(ButtonStyle::Success)
            .label(format!("Pay {money}")),
        CreateButton::new(format!("{REQUEST_DECLINE}{id}"))
            .style(ButtonStyle::Secondary)
            .label("Decline"),
    ])
}

/// Moves a request between statuses, `None` if it wasn't in `from` (already answered) or the lookup failed.
/// Only the payer it was made to can move it.
async fn transition(
    id: uuid::Uuid,
    payer: UserId,
    from: BankRequestStatus,
    to: BankRequestStatus,
) -> Option<BankRequest> {
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        TransitionBankRequest {
            id,
            payer,
            from,
            to,
        }
    )
    .await
    {
        Ok(Ok((Response::TransitionBankRequest(Ok(request)), _))) => request,
        other => {
            error!("failed to move bank request {id} from {from:?} to {to:?}: {other:?}");
            None
        }
    }
}

/// Handles the Pay and Decline buttons of a `/bank request`, pressed by the member asked to pay
#[allow(clippy::too_many_lines)]
pub async fn request_button(
    ctx: &Context,
    component: &ComponentInteraction,
) -> serenity::Result<()> {
    let id = component.data.custom_id.as_str();
    let (paying, rest) = if let Some(rest) = id.strip_prefix(REQUEST_PAY) {
        (true, rest)
    } else if let Some(rest) = id.strip_prefix(REQUEST_DECLINE) {
        (false, rest)
    } else {
        return Ok(());
    };
    let Ok(request_id) = uuid::Uuid::parse_str(rest) else {
        // Buttons from before requests were saved, which can't be looked up
        return ephemeral(
            ctx,
            component,
            "This request is from before an update. Ask them to send a new one.",
        )
        .await;
    };
    let payer = component.user.id;

    if !paying {
        let Some(request) = transition(
            request_id,
            payer,
            BankRequestStatus::Open,
            BankRequestStatus::Declined,
        )
        .await
        else {
            return ephemeral(ctx, component, "This request has already been answered.").await;
        };
        component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .content(format!("{}\n\n:x: Declined", component.message.content))
                        .components(vec![]),
                ),
            )
            .await?;
        dm(
            ctx,
            request.requester,
            format!(
                "<@{payer}> declined your request for {}\n> {}",
                bootstrap::format::money(request.amount, false),
                request.reason
            ),
        )
        .await;
        return Ok(());
    }

    // The same rule as /bank transfer, which can also ask staff for permission
    let active = GUILD
        .member(&ctx, payer)
        .await
        .is_ok_and(|member| member.roles.contains(&ACTIVE));
    if !active {
        return ephemeral(
            ctx,
            component,
            "You need to be an active member to send money. Use `/bank transfer` to ask staff for permission.",
        )
        .await;
    }

    // Claimed before any money moves, so the request can't be paid twice
    let Some(request) = transition(
        request_id,
        payer,
        BankRequestStatus::Open,
        BankRequestStatus::Paying,
    )
    .await
    else {
        return ephemeral(ctx, component, "This request has already been answered.").await;
    };
    let money = bootstrap::format::money(request.amount, false);
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "{}\n\n:hourglass: Paying {money}...",
                        component.message.content
                    ))
                    .components(vec![]),
            ),
        )
        .await?;

    let outcome = if send_money(
        ctx,
        payer,
        request.requester,
        request.amount,
        &request.reason,
    )
    .await
    .is_some()
    {
        transition(
            request_id,
            payer,
            BankRequestStatus::Paying,
            BankRequestStatus::Paid,
        )
        .await;
        (":white_check_mark: Paid", vec![])
    } else {
        // Back to open, with the buttons, so it can be tried again
        transition(
            request_id,
            payer,
            BankRequestStatus::Paying,
            BankRequestStatus::Open,
        )
        .await;
        (
            ":warning: The payment failed, nothing was sent. You can try again.",
            vec![request_buttons(request_id, &money)],
        )
    };
    component
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .content(format!("{}\n\n{}", component.message.content, outcome.0))
                .components(outcome.1),
        )
        .await?;
    Ok(())
}

async fn ephemeral(
    ctx: &Context,
    component: &ComponentInteraction,
    content: &str,
) -> serenity::Result<()> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true),
            ),
        )
        .await
}
