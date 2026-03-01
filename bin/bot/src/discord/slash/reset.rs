use crate::{audit, discord::interaction::Interaction, get_option};
use serenity::{
    all::{
        CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType, CreateAutocompleteResponse, CreateInteractionResponse,
    },
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::reset::db::Response;
use synixe_proc::events_request_2;
use uuid::Uuid;
pub fn register() -> CreateCommand {
    CreateCommand::new("reset")
        .description("Here for a limited time only!")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "kit", "Receive a kit")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "first_kit",
                        "Kit to receieve",
                    )
                    .set_autocomplete(true)
                    .required(true),
                ),
        )
}
pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "kit" => kit(ctx, command, options).await?,
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
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "kit" => kit_autocomplete(ctx, autocomplete, options).await?,
            _ => return Ok(()),
        }
    }
    Ok(())
}
#[allow(clippy::too_many_lines)]
async fn kit(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Some(first_kit) = get_option!(options, "first_kit", String) else {
        return interaction
            .reply("Required option not provided: first_kit")
            .await;
    };
    if let Ok(Ok((Response::CanClaim(Ok(Some(Some(false)))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        CanClaim {
            member: command.user.id,
            first_kit: Uuid::parse_str(first_kit).expect("first_kit should be a uuid")
        }
    )
    .await
    {
        if let Ok(Ok((Response::LastClaim(Ok(Some(last_claim))), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::reset::db,
            LastClaim {
                member: command.user.id,
            }
        )
        .await
        {
            let last_claim = last_claim.to_utc().unix_timestamp();
            audit(format!(
                "Member <@{}> attempted to claim a reset kit but is on cooldown. Last claim was at <t:{}:F> (<t:{}:R>)",
                command.user.id, last_claim, last_claim
            ));
            interaction
                .reply(format!("Only one reset kit can be claimed every 14 days. Your last claim was at <t:{last_claim}:F> (<t:{last_claim}:R>)"))
                .await?;
            return Ok(());
        }
        interaction
            .reply("Only one reset kit can be claimed every 14 days.")
            .await?;
        return Ok(());
    }
    let Ok(Ok((Response::ClaimKit(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        ClaimKit {
            member: command.user.id,
            first_kit: Uuid::parse_str(first_kit).expect("first_kit should be a uuid")
        }
    )
    .await
    else {
        error!("Failed to claim kit");
        return interaction
            .reply("Failed to claim kit, please try again later")
            .await;
    };
    interaction.reply("Kit claimed!").await?;
    audit(format!(
        "Member <@{}> claimed reset kit for first kit {}",
        command.user.id, first_kit
    ));
    Ok(())
}
async fn kit_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
    _options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name != "first_kit" {
        return Ok(());
    }
    let Ok(Ok((Response::UnclaimedKits(Ok(first_kits)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        UnclaimedKits {
            member: autocomplete.user.id
        }
    )
    .await
    else {
        error!("Failed to fetch unclaimed kits");
        return Ok(());
    };
    let mut first_kits: Vec<_> = first_kits
        .into_iter()
        .filter(|c| {
            c.name
                .to_lowercase()
                .contains(&focus.value.to_string().to_lowercase())
        })
        .collect();
    if first_kits.len() > 25 {
        first_kits.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for kit in first_kits {
                f = f.add_string_choice(&kit.name, kit.id.to_string());
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
