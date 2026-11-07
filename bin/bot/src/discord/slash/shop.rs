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

    let name = pretty.as_deref().unwrap_or(class);
    let sale = price
        .until()
        .map(|until| format!("\nSale ends: {}", until.date()))
        .unwrap_or_default();
    interaction
        .reply(format!(
            "**{name}**\nPersonal: {}\nCompany: {}{sale}",
            bootstrap::format::money(price.personal(), false),
            bootstrap::format::money(price.company(), false),
        ))
        .await
}
