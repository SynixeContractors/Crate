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
use synixe_meta::discord::role::CERT_GRENADIER;
use synixe_proc::events_request_2;

use crate::{
    discord::interaction::{Confirmation, Interaction},
    get_option,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("gear")
        .description("Manage your gear")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "repaint", "Repaint a weapon")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "item",
                        "Select the item you want to modify",
                    )
                    .required(true)
                    .set_autocomplete(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "color",
                        "Select the new variant of the item",
                    )
                    .required(true)
                    .set_autocomplete(true),
                ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "ugl",
                "Upgrade a weapon to have a UGL",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "weapon",
                    "Select the weapon you want to modify",
                )
                .required(true)
                .set_autocomplete(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "ugl",
                    "Select the UGL you want to add",
                )
                .required(true)
                .set_autocomplete(true),
            ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "repaint" => repaint(ctx, command, options).await?,
            "ugl" => ugl(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if subcommand.kind() == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "repaint" => repaint_autocomplete(ctx, autocomplete).await?,
            "ugl" => ugl_autocomplete(ctx, autocomplete).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn repaint(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Some(item) = get_option!(options, "item", String) else {
        return interaction.reply("Invalid item").await;
    };
    let Some(color) = get_option!(options, "color", String) else {
        return interaction.reply("Invalid color").await;
    };
    let Ok(Ok((Response::FamilySearch(Ok(family_items)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        FamilySearch {
            item: item.clone(),
            relation: "color".to_string(),
        }
    )
    .await
    else {
        error!("failed to fetch item list");
        return Ok(());
    };
    let mut original_name = None;
    let mut new_name = None;
    for family_item in family_items {
        if &family_item.class == color {
            new_name = Some(family_item.pretty);
        } else if &family_item.class == item {
            original_name = Some(family_item.pretty);
        }
    }
    if new_name.is_none() || original_name.is_none() {
        return interaction
            .reply("Invalid selections, not a valid variant")
            .await;
    }
    if interaction
        .confirm(&format!(
            "Are you sure you want to repaint {} to {} for $150?",
            original_name.expect("name exists, checked for invalid selection"),
            new_name.expect("name exists, checked for invalid selection")
        ))
        .await?
        == Confirmation::Yes
    {
        let Ok(Ok((Response::FamilyReplace(Ok(())), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyReplace {
                member: command.user.id,
                original: item.clone(),
                new: color.clone(),
                reason: "color".to_string(),
                cost: 150,
            }
        )
        .await
        else {
            error!("failed to repaint item");
            return Ok(());
        };
        interaction.reply("Repaint Complete!").await?;
    } else {
        interaction.reply("Repaint cancelled").await?;
    }
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn repaint_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name == "item" {
        let Ok(Ok((Response::FamilyCompatibleItems(Ok(mut items)), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyCompatibleItems {
                member: autocomplete.user.id,
                relation: "color".to_string(),
            }
        )
        .await
        else {
            error!("failed to fetch item list");
            return Ok(());
        };
        items.truncate(25);
        if let Err(e) = autocomplete
            .create_response(&ctx.http, {
                let mut f = CreateAutocompleteResponse::default();
                for item in items {
                    f = f.add_string_choice(&item.pretty, &item.class);
                }
                CreateInteractionResponse::Autocomplete(f)
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    } else if focus.name == "color" {
        let CommandDataOptionValue::SubCommand(options) = &autocomplete.data.options[0].value
        else {
            return Ok(());
        };
        let item = options
            .iter()
            .find(|o| o.name == "item")
            .and_then(|o| o.value.as_str())
            .unwrap_or_default()
            .to_string();
        let Ok(Ok((Response::FamilySearch(Ok(mut colors)), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilySearch {
                item: item.clone(),
                relation: "color".to_string(),
            }
        )
        .await
        else {
            error!("failed to fetch item list");
            return Ok(());
        };
        colors.truncate(25);
        if let Err(e) = autocomplete
            .create_response(&ctx.http, {
                let mut f = CreateAutocompleteResponse::default();
                for color in colors {
                    if color.class == item {
                        continue;
                    }
                    f = f.add_string_choice(&color.pretty, &color.class);
                }
                CreateInteractionResponse::Autocomplete(f)
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    }
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn ugl(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    // Check if they have the Grenadier role
    let Some(guild) = command.guild_id else {
        return interaction
            .reply("This command must be used in a server")
            .await;
    };
    let member = guild
        .member(&ctx, command.user.id)
        .await
        .expect("User must be in Synixe");
    if !member.roles.contains(&CERT_GRENADIER) {
        return interaction
            .reply("You must be a Grenadier to use this command")
            .await;
    }
    let Some(weapon) = get_option!(options, "weapon", String) else {
        return interaction.reply("Invalid weapon").await;
    };
    let Some(ugl) = get_option!(options, "ugl", String) else {
        return interaction.reply("Invalid UGL").await;
    };
    let Ok(Ok((Response::FamilySearch(Ok(family_items)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        FamilySearch {
            item: weapon.clone(),
            relation: "ugl".to_string(),
        }
    )
    .await
    else {
        error!("failed to fetch item list");
        return Ok(());
    };
    let mut original_name = None;
    let mut new_name = None;
    for family_item in family_items {
        if &family_item.class == ugl {
            new_name = Some(family_item.pretty);
        } else if &family_item.class == weapon {
            original_name = Some(family_item.pretty);
        }
    }
    if new_name.is_none() || original_name.is_none() {
        return interaction
            .reply("Invalid selections, not a valid variant")
            .await;
    }
    if interaction
        .confirm(&format!(
            "Are you sure you want to upgrade {} to {} for $300?",
            original_name.expect("name exists, checked for invalid selection"),
            new_name.expect("name exists, checked for invalid selection")
        ))
        .await?
        == Confirmation::Yes
    {
        let Ok(Ok((Response::FamilyReplace(Ok(())), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyReplace {
                member: command.user.id,
                original: weapon.clone(),
                new: ugl.clone(),
                reason: "ugl".to_string(),
                cost: 300,
            }
        )
        .await
        else {
            error!("failed to repaint item");
            return Ok(());
        };
        interaction.reply("Upgrade Complete!").await?;
    } else {
        interaction.reply("Upgrade cancelled").await?;
    }
    Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn ugl_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name == "weapon" {
        let Ok(Ok((Response::FamilyCompatibleItems(Ok(mut items)), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyCompatibleItems {
                member: autocomplete.user.id,
                relation: "ugl".to_string(),
            }
        )
        .await
        else {
            error!("failed to fetch item list");
            return Ok(());
        };
        items.truncate(25);
        if let Err(e) = autocomplete
            .create_response(&ctx.http, {
                let mut f = CreateAutocompleteResponse::default();
                for item in items {
                    if item.class != item.family {
                        continue;
                    }
                    f = f.add_string_choice(&item.pretty, &item.class);
                }
                CreateInteractionResponse::Autocomplete(f)
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    } else if focus.name == "ugl" {
        let CommandDataOptionValue::SubCommand(options) = &autocomplete.data.options[0].value
        else {
            return Ok(());
        };
        let weapon = options
            .iter()
            .find(|o| o.name == "weapon")
            .and_then(|o| o.value.as_str())
            .unwrap_or_default()
            .to_string();
        let Ok(Ok((Response::FamilySearch(Ok(mut ugls)), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilySearch {
                item: weapon.clone(),
                relation: "ugl".to_string(),
            }
        )
        .await
        else {
            error!("failed to fetch item list");
            return Ok(());
        };
        ugls.truncate(25);
        if let Err(e) = autocomplete
            .create_response(&ctx.http, {
                let mut f = CreateAutocompleteResponse::default();
                for ugl in ugls {
                    if ugl.class == weapon {
                        continue;
                    }
                    f = f.add_string_choice(&ugl.pretty, &ugl.class);
                }
                CreateInteractionResponse::Autocomplete(f)
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    }
    Ok(())
}
