use serenity::{
    all::{
        CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType,
    },
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    },
    client::Context,
};
use synixe_events::gear::db::Response;
use synixe_proc::events_request_5;

use crate::{discord::interaction::Interaction, get_option};

use super::AllowPublic;

pub fn register() -> CreateCommand {
    CreateCommand::new("shop")
        .description("Interact with the shop")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "price",
                "View the price of an item in the shop",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "item",
                    "The item's name or class name",
                )
                .set_autocomplete(true)
                .required(true),
            )
            .allow_public(),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for shop provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "price" => price(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };

    let Ok(Ok((Response::ShopSearch(Ok(items)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        ShopSearch {
            search: focus.value.to_string()
        }
    )
    .await
    else {
        error!("failed to fetch shop items for autocomplete");
        return Ok(());
    };

    let mut choices = CreateAutocompleteResponse::default();
    for (class, pretty) in items {
        let name: String = pretty
            .as_deref()
            .unwrap_or(&class)
            .chars()
            .take(100)
            .collect();
        choices = choices.add_string_choice(name, class);
    }
    if let Err(e) = autocomplete
        .create_response(&ctx.http, CreateInteractionResponse::Autocomplete(choices))
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn price(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching price...").await?;
    let Some(input) = get_option!(options, "item", String) else {
        return interaction.reply("Invalid item").await;
    };

    let Ok(Ok((Response::ShopSearch(Ok(matches)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        ShopSearch {
            search: input.clone()
        }
    )
    .await
    else {
        return interaction.reply("Failed to search the shop").await;
    };

    // Exact matches are sorted first, so a single unambiguous result can be used too
    let found = matches
        .iter()
        .find(|(class, pretty)| {
            class.eq_ignore_ascii_case(input)
                || pretty
                    .as_deref()
                    .is_some_and(|p| p.eq_ignore_ascii_case(input))
        })
        .or_else(|| matches.first().filter(|_| matches.len() == 1));
    let Some((class, pretty)) = found else {
        return interaction
            .reply("No item found. Pick one from the autocomplete list")
            .await;
    };

    let Ok(Ok((Response::ShopGetPrice(Ok(price)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        ShopGetPrice {
            item: class.clone()
        }
    )
    .await
    else {
        return interaction.reply("Failed to fetch price").await;
    };

    let history = match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        ShopPriceHistory {
            item: class.clone()
        }
    )
    .await
    {
        Ok(Ok((Response::ShopPriceHistory(Ok(history)), _))) => {
            sparkline(&history, time::OffsetDateTime::now_utc())
                .map_or_else(String::new, |line| format!("\n{line}"))
        }
        _ => String::new(),
    };

    let name = pretty.as_deref().unwrap_or(class);
    let sale = price.until().map_or_else(String::new, |until| {
        format!("\nSale ends: {}", until.date())
    });
    interaction
        .reply(format!(
            "**{name}**\nPersonal: {}\nCompany: {}{sale}{history}",
            bootstrap::format::money(price.personal(), false),
            bootstrap::format::money(price.company(), false),
        ))
        .await
}

/// Weeks of price history shown
const HISTORY_WEEKS: i64 = 26;
const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// What members paid each week, from weekly averages (oldest first), as a sparkline over the last
/// `HISTORY_WEEKS`. Weeks without purchases carry the last price forward. `None` without two weeks to compare.
fn sparkline(history: &[(time::OffsetDateTime, i32)], now: time::OffsetDateTime) -> Option<String> {
    let start = now - time::Duration::weeks(HISTORY_WEEKS - 1);
    let mut prices = Vec::new();
    let mut known = history
        .iter()
        .take_while(|(week, _)| *week < start)
        .last()
        .map(|(_, price)| *price);
    let mut points = history
        .iter()
        .skip_while(|(week, _)| *week < start)
        .peekable();
    for week in 0..HISTORY_WEEKS {
        let end = start + time::Duration::weeks(week + 1);
        while let Some((_, price)) = points.next_if(|(at, _)| *at < end) {
            known = Some(*price);
        }
        if let Some(price) = known {
            prices.push(price);
        }
    }
    if prices.len() < 2 {
        return None;
    }
    let low = *prices.iter().min()?;
    let high = *prices.iter().max()?;
    let line = prices
        .iter()
        .map(|price| {
            if high == low {
                SPARKS[0]
            } else {
                let steps = i64::try_from(SPARKS.len() - 1).unwrap_or(0);
                let level = i64::from(price - low) * steps / i64::from(high - low);
                SPARKS[usize::try_from(level).unwrap_or(0)]
            }
        })
        .collect::<String>();
    let money = |price: i32| bootstrap::format::money(price, false);
    Some(format!(
        "Paid over {} weeks: `{line}` {} → {} (low {}, high {})",
        prices.len(),
        money(*prices.first()?),
        money(*prices.last()?),
        money(low),
        money(high),
    ))
}

#[cfg(test)]
mod tests {
    use time::{Duration, macros::datetime};

    use super::*;

    #[test]
    fn needs_two_weeks() {
        let now = datetime!(2026-11-24 12:00 UTC);
        assert_eq!(sparkline(&[], now), None);
        assert_eq!(sparkline(&[(now, 100)], now), None);
    }

    #[test]
    fn rises_and_carries_forward() {
        let now = datetime!(2026-11-24 12:00 UTC);
        let history = [
            (now - Duration::weeks(3), 100),
            (now - Duration::weeks(1), 200),
        ];
        let line = sparkline(&history, now).expect("enough history");
        // Four weeks: 100, 100 carried forward, 200, 200 carried forward
        assert!(line.contains("`▁▁██`"), "{line}");
        assert!(line.contains("Paid over 4 weeks"), "{line}");
    }

    #[test]
    fn older_price_starts_the_line() {
        let now = datetime!(2026-11-24 12:00 UTC);
        let history = [(now - Duration::weeks(40), 100)];
        let line = sparkline(&history, now).expect("carried forward");
        assert!(
            line.contains(&format!("Paid over {HISTORY_WEEKS} weeks")),
            "{line}"
        );
    }
}
