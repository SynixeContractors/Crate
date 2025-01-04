use crate::{discord::interaction::Interaction, get_option};
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
                        "certification",
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
    if let Ok(Ok((Response::CanClaim(Ok(Some(Some(false)))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        CanClaim {
            member: command.user.id
        }
    )
    .await
    {
        interaction
            .reply("You can not claim a kit at this time")
            .await?;
        return Ok(());
    }
    let Some(cert) = get_option!(options, "certification", String) else {
        return interaction
            .reply("Required option not provided: certification")
            .await;
    };
    let Ok(Ok((Response::ClaimKit(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        ClaimKit {
            member: command.user.id,
            cert: Uuid::parse_str(cert).expect("certification should be a uuid")
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
    if focus.name != "certification" {
        return Ok(());
    }
    let Ok(Ok((Response::UnclaimedKits(Ok(certs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::reset::db,
        UnclaimedKits {
            member: autocomplete.user.id
        }
    )
    .await
    else {
        error!("Failed to fetch certifications");
        return Ok(());
    };
    let mut certs: Vec<_> = certs
        .into_iter()
        .filter(|c| {
            c.name
                .to_lowercase()
                .contains(&focus.value.to_string().to_lowercase())
        })
        .collect();
    if certs.len() > 25 {
        certs.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for cert in certs {
                f = f.add_string_choice(&cert.name, cert.id.to_string());
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
