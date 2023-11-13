use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::gear::db::Response;
use synixe_meta::discord::role::CERT_GRENADIER;
use synixe_proc::events_request_2;

use crate::{
    discord::interaction::{Confirmation, Generic, Interaction},
    get_option,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("gear")
        .description("Manage your gear")
        .create_option(|option| {
            option
                .name("repaint")
                .description("Repaint a weapon")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("item")
                        .description("Select the item you want to modify")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("color")
                        .description("Select the new variant of the item")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
        .create_option(|option| {
            option
                .name("ugl")
                .description("Upgrade a weapon to have a UGL")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("weapon")
                        .description("Select the weapon you want to modify")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("ugl")
                        .description("Select the UGL you want to add")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "repaint" => repaint(ctx, command, &subcommand.options).await?,
            "ugl" => ugl(ctx, command, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
) -> serenity::Result<()> {
    let Some(subcommand) = autocomplete.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "repaint" => repaint_autocomplete(ctx, autocomplete, &subcommand.options).await?,
            "ugl" => ugl_autocomplete(ctx, autocomplete, &subcommand.options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn repaint(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
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

async fn repaint_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = options.iter().find(|o| o.focused) else {
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
        if items.len() > 25 {
            items.truncate(25);
        }
        if let Err(e) = autocomplete
            .create_autocomplete_response(&ctx.http, |f| {
                for item in items {
                    f.add_string_choice(&item.pretty, &item.class);
                }
                f
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
        Ok(())
    } else if focus.name == "color" {
        let item = autocomplete.data.options[0]
            .options
            .iter()
            .find(|o| o.name == "item")
            .and_then(|o| o.value.as_ref().map(|v| v.as_str().unwrap_or_default()))
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
        if colors.len() > 25 {
            colors.truncate(25);
        }
        if let Err(e) = autocomplete
            .create_autocomplete_response(&ctx.http, |f| {
                for color in colors {
                    if color.class == item {
                        continue;
                    }
                    f.add_string_choice(&color.pretty, &color.class);
                }
                f
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
        Ok(())
    } else {
        Ok(())
    }
}

async fn ugl(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    // Check if they have the Grenadier role
    let guild = command
        .guild_id
        .expect("Brodsky is only in Synixe")
        .to_guild_cached(ctx)
        .expect("Brodsky is only in Synixe");
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

async fn ugl_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = options.iter().find(|o| o.focused) else {
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
        if items.len() > 25 {
            items.truncate(25);
        }
        if let Err(e) = autocomplete
            .create_autocomplete_response(&ctx.http, |f| {
                for item in items {
                    if item.class != item.family {
                        continue;
                    }
                    f.add_string_choice(&item.pretty, &item.class);
                }
                f
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    } else if focus.name == "ugl" {
        let weapon = autocomplete.data.options[0]
            .options
            .iter()
            .find(|o| o.name == "weapon")
            .and_then(|o| o.value.as_ref().map(|v| v.as_str().unwrap_or_default()))
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
        if ugls.len() > 25 {
            ugls.truncate(25);
        }
        if let Err(e) = autocomplete
            .create_autocomplete_response(&ctx.http, |f| {
                for ugl in ugls {
                    if ugl.class == weapon {
                        continue;
                    }
                    f.add_string_choice(&ugl.pretty, &ugl.class);
                }
                f
            })
            .await
        {
            error!("failed to create autocomplete response: {}", e);
        }
    }
    Ok(())
}
