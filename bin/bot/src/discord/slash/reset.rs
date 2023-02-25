use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{ApplicationCommandInteraction, CommandDataOption},
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::reset::db::Response;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("reset")
        .description("Here for a limited time only!")
        .create_option(|option| {
            option
                .name("kit")
                .description("Receive a free kit")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("certification")
                        .description("Kit to receieve")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "kit" => kit(ctx, command, &subcommand.options).await?,
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
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "kit" {
        kit_autocomplete(ctx, autocomplete, &subcommand.options).await?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn kit(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
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
    ).await else {
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
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(focus) = options.iter().find(|o| o.focused) else {
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
    .await else {
        error!("Failed to fetch certifications");
        return Ok(());
    };
    let mut certs: Vec<_> = certs
        .into_iter()
        .filter(|c| {
            c.name.to_lowercase().contains(
                &focus
                    .value
                    .as_ref()
                    .expect("focused option should always have a value")
                    .as_str()
                    .expect("value should always be a string")
                    .to_lowercase(),
            )
        })
        .collect();
    if certs.len() > 25 {
        certs.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_autocomplete_response(&ctx.http, |f| {
            for cert in certs {
                f.add_string_choice(&cert.name, cert.id);
            }
            f
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}
