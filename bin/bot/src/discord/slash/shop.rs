use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
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
                    "The item's class name",
                )
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

async fn price(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching price...").await?;
    let Some(item) = get_option!(options, "item", String) else {
        return interaction.reply("Invalid item").await;
    };

    let Ok(Ok((Response::ShopGetPrice(Ok(price)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        ShopGetPrice { item: item.clone() }
    )
    .await
    else {
        return interaction
            .reply("Failed to fetch price, check the item's class name")
            .await;
    };

    let sale = price
        .until()
        .map(|until| format!("\nSale ends: {}", until.date()))
        .unwrap_or_default();
    interaction
        .reply(format!(
            "**{item}**\nPersonal: {}\nCompany: {}{sale}",
            bootstrap::format::money(price.personal(), false),
            bootstrap::format::money(price.company(), false),
        ))
        .await
}
