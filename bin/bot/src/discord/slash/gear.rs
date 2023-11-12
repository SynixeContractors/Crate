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
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "repaint" => repaint(ctx, command, &subcommand.options).await?,
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
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "repaint" {
        repaint_autocomplete(ctx, autocomplete, &subcommand.options).await?;
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
    let Ok(Ok((Response::FamilySearch(Ok(colors)), _))) = events_request_2!(
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
    for (variant, name) in colors {
        if &variant == color {
            new_name = Some(name);
        } else if &variant == item {
            original_name = Some(name);
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
        let Ok(Ok((Response::FamilyRepaint(Ok(())), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyRepaint {
                member: command.user.id,
                original: item.clone(),
                new: color.clone(),
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
                    f.add_string_choice(&item.1, &item.0);
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
                    if color.0 == item {
                        continue;
                    }
                    f.add_string_choice(&color.1, &color.0);
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
