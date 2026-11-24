use std::fmt::Write;

use serenity::{
    all::{
        CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType, CreateAttachment, RoleId,
    },
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    },
    client::Context,
};
use synixe_events::gear::db::Response;
use synixe_meta::discord::role::CERT_GRENADIER;
use synixe_proc::events_request_5;

use crate::{
    discord::interaction::{Confirmation, Interaction},
    get_option, get_option_user,
};

use super::AllowPublic;

/// Configuration for a gear modification command
struct GearCommandConfig {
    /// Database relation type and command (e.g., "color", "ugl")
    relation: &'static str,
    /// User-facing description
    description: &'static str,
    /// Cost in currency
    cost: i32,
    /// Name of the item/source option
    item_option: &'static str,
    /// Name of the variant/target option
    variant_option: &'static str,
    /// Whether to filter items where class != family (for base items only)
    filter_base_only: bool,
    /// Required roles to use this command (empty slice means no role requirement)
    required_roles: &'static [RoleId],
}

/// All available gear modification commands
const GEAR_COMMANDS: &[GearCommandConfig] = &[
    GearCommandConfig {
        relation: "color",
        description: "Repaint a weapon",
        cost: 150,
        item_option: "item",
        variant_option: "color",
        filter_base_only: false,
        required_roles: &[],
    },
    GearCommandConfig {
        relation: "ugl",
        description: "Upgrade a weapon to have a UGL",
        cost: 300,
        item_option: "weapon",
        variant_option: "ugl",
        filter_base_only: true,
        required_roles: &[CERT_GRENADIER],
    },
    GearCommandConfig {
        relation: "belt",
        description: "Add a belt to a vest",
        cost: 100,
        item_option: "current",
        variant_option: "new",
        filter_base_only: true,
        required_roles: &[],
    },
    GearCommandConfig {
        relation: "barrel",
        description: "Change your weapon barrel",
        cost: 60,
        item_option: "weapon",
        variant_option: "barrel",
        filter_base_only: false,
        required_roles: &[],
    },
];

pub fn register() -> CreateCommand {
    let mut cmd = CreateCommand::new("gear").description("Manage your gear");

    for config in GEAR_COMMANDS {
        let sub = CreateCommandOption::new(
            CommandOptionType::SubCommand,
            config.relation,
            config.description,
        )
        .add_sub_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                config.item_option,
                format!("Select the {} you want to modify", config.item_option),
            )
            .required(true)
            .set_autocomplete(true),
        )
        .add_sub_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                config.variant_option,
                format!("Select the new {} of the item", config.variant_option),
            )
            .required(true)
            .set_autocomplete(true),
        );

        cmd = cmd.add_option(sub);
    }

    cmd.add_option(
        CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "locker",
            "View the items in a member's locker",
        )
        .add_sub_option(CreateCommandOption::new(
            CommandOptionType::User,
            "member",
            "The member to view the locker of, yourself if not given",
        ))
        .allow_public(),
    )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for gear provided");
        return Ok(());
    };

    let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
        return Ok(());
    };
    if subcommand.name == "locker" {
        return locker(ctx, command, options).await;
    }
    if let Some(config) = GEAR_COMMANDS.iter().find(|c| c.relation == subcommand.name) {
        run_gear_command(ctx, command, options, config).await?;
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

    if subcommand.kind() == CommandOptionType::SubCommand
        && let Some(config) = GEAR_COMMANDS.iter().find(|c| c.relation == subcommand.name)
    {
        gear_autocomplete(ctx, autocomplete, config).await?;
    }
    Ok(())
}

async fn run_gear_command(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
    config: &GearCommandConfig,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);

    // Check required roles if specified
    if !config.required_roles.is_empty() {
        let Some(guild) = command.guild_id else {
            return interaction
                .reply("This command must be used in a server")
                .await;
        };

        let member = guild
            .member(&ctx, command.user.id)
            .await
            .expect("User must be in Synixe");

        let has_required_role = config
            .required_roles
            .iter()
            .any(|role| member.roles.contains(role));
        if !has_required_role {
            return interaction
                .reply("You do not have the required role for this command")
                .await;
        }
    }

    let Some(item) = get_option!(options, config.item_option, String) else {
        return interaction.reply("Invalid item").await;
    };

    let Some(variant) = get_option!(options, config.variant_option, String) else {
        return interaction.reply("Invalid variant").await;
    };

    let Ok(Ok((Response::FamilySearch(Ok(family_items)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        FamilySearch {
            item: item.clone(),
            relation: config.relation.to_string(),
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
        if family_item.class == *variant {
            new_name = Some(family_item.pretty);
        } else if family_item.class == *item {
            original_name = Some(family_item.pretty);
        }
    }

    let (Some(original_name), Some(new_name)) = (original_name, new_name) else {
        return interaction
            .reply("Invalid selections, not a valid variant")
            .await;
    };

    if interaction
        .confirm(&format!(
            "Are you sure you want to change {original_name} to {new_name} for ${}?",
            config.cost
        ))
        .await?
        == Confirmation::Yes
    {
        match events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyReplace {
                member: command.user.id,
                original: item.clone(),
                new: variant.clone(),
                reason: config.relation.to_string(),
                cost: config.cost,
            }
        )
        .await
        {
            Ok(Ok((Response::FamilyReplace(Ok(())), _))) => {}
            // The database's reason is written to be shown, e.g. insufficient funds
            Ok(Ok((Response::FamilyReplace(Err(reason)), _))) => {
                return interaction.reply(reason).await;
            }
            _ => {
                error!("failed to modify gear");
                return interaction
                    .reply("Failed to modify gear, please try again later")
                    .await;
            }
        }

        interaction.reply("Modification complete!").await?;
    } else {
        interaction.reply("Modification cancelled").await?;
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
async fn gear_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
    config: &GearCommandConfig,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };

    if focus.name == config.item_option {
        let Ok(Ok((Response::FamilyCompatibleItems(Ok(mut items)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilyCompatibleItems {
                member: autocomplete.user.id,
                relation: config.relation.to_string(),
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
                    // Filter to base items only if configured
                    if config.filter_base_only && item.class != item.family {
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
    } else if focus.name == config.variant_option {
        let CommandDataOptionValue::SubCommand(options) = &autocomplete.data.options[0].value
        else {
            return Ok(());
        };

        let item = options
            .iter()
            .find(|o| o.name == config.item_option)
            .and_then(|o| o.value.as_str())
            .unwrap_or_default()
            .to_string();

        let Ok(Ok((Response::FamilySearch(Ok(mut variants)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            FamilySearch {
                item: item.clone(),
                relation: config.relation.to_string(),
            }
        )
        .await
        else {
            error!("failed to fetch variant list");
            return Ok(());
        };

        variants.truncate(25);

        if let Err(e) = autocomplete
            .create_response(&ctx.http, {
                let mut f = CreateAutocompleteResponse::default();
                for variant in variants {
                    // Don't suggest the same item they already have
                    if variant.class == item {
                        continue;
                    }
                    f = f.add_string_choice(&variant.pretty, &variant.class);
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

/// A locker longer than this is sent as a file, keeping the message under Discord's 2000 characters
const LOCKER_MESSAGE_LENGTH: usize = 1800;
/// The most characters of an item's name shown
const LOCKER_NAME_LENGTH: usize = 40;

async fn locker(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching locker...").await?;
    let user = get_option_user!(options, "member")
        .copied()
        .unwrap_or(command.user.id);

    let Ok(Ok((Response::LockerContents(Ok(items)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerContents { member: user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch the locker").await;
    };
    if items.is_empty() {
        return interaction
            .reply(format!("<@{user}>'s locker is empty"))
            .await;
    }

    let mut total = 0;
    let mut list = String::new();
    for item in &items {
        let name: String = item
            .pretty
            .as_deref()
            .unwrap_or(&item.class)
            .chars()
            .take(LOCKER_NAME_LENGTH)
            .collect();
        let value = item.value.map(|each| each * item.quantity);
        total += value.unwrap_or(0);
        let _ = writeln!(
            list,
            "{:>4}x  {name:<LOCKER_NAME_LENGTH$}  {:>10}",
            item.quantity,
            value.map_or_else(
                || String::from("-"),
                |value| bootstrap::format::money(value, false)
            ),
        );
    }
    let summary = format!(
        "<@{user}>'s locker: {} items worth {}",
        items.iter().map(|item| item.quantity).sum::<i32>(),
        bootstrap::format::money(total, false)
    );
    if list.len() > LOCKER_MESSAGE_LENGTH {
        interaction
            .reply_with_attachment(
                summary,
                CreateAttachment::bytes(list.into_bytes(), "locker.txt"),
            )
            .await
    } else {
        interaction.reply(format!("{summary}\n```{list}```")).await
    }
}
