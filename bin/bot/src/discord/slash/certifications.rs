use std::collections::HashSet;

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
use synixe_events::certifications::db::Response;
use synixe_meta::discord::{
    role::{JUNIOR, MEMBER},
    GUILD,
};
use synixe_proc::events_request_2;

use crate::{discord::interaction::Interaction, get_option, get_option_user};

use super::AllowPublic;

pub fn register() -> CreateCommand {
    CreateCommand::new("certifications")
        .description("Certifications")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "trial",
                "You ran someone through a certification trial",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "trainee",
                    "The person you ran through the trial",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "certification",
                    "The certification you ran them through",
                )
                .required(true)
                .set_autocomplete(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Boolean,
                    "passed",
                    "Did the trainee pass the trial?",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "notes",
                    "Notes about the trial, only shared between you and the trainee",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "view",
                "View someone's certifications",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    "member",
                    "The member to view certifications for",
                )
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "List all certifications",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "available",
                "List all certifications available to you",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::User,
                "member",
                "The member to view certifications for",
            ))
            .allow_public(),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(values) = &subcommand.value {
        match subcommand.name.as_str() {
            "trial" => trial(ctx, command, values).await?,
            "view" => view(ctx, command, values).await?,
            "list" => list(ctx, command, values, false).await?,
            "available" => list(ctx, command, values, true).await?,
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
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind() == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "trial" => trial_autocomplete(ctx, autocomplete).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn trial(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching certifications...").await?;
    let Ok(Ok((Response::ListInstructor(Ok(certs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
            member: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id
        }
    )
    .await
    else {
        return interaction.reply("Failed to fetch certifications").await;
    };
    if certs.is_empty() {
        return interaction
            .reply("You are not an instructor for any certifications")
            .await;
    }
    let Some(cert) = get_option!(options, "certification", String) else {
        return interaction.reply("Invalid certification").await;
    };
    let Some(cert) = certs.iter().find(|c| &c.id.to_string() == cert) else {
        return interaction.reply("Invalid certification").await;
    };
    let Some(passed) = get_option!(options, "passed", Boolean) else {
        return interaction.reply("Invalid passed").await;
    };
    let Some(trainee) = get_option_user!(options, "trainee") else {
        return interaction.reply("Invalid trainee").await;
    };
    let Some(notes) = get_option!(options, "notes", String) else {
        return interaction.reply("Invalid notes").await;
    };
    interaction.reply("Submitting trial...").await?;
    match events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Certify {
            instructor: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id,
            trainee: *trainee,
            certification: cert.id,
            notes: notes.clone(),
            passed: *passed,
        }
    )
    .await
    {
        Err(e) | Ok(Ok((Response::Certify(Err(e)), _))) => {
            interaction
                .reply(format!("Failed to submit trial: {e}"))
                .await?;
        }
        Ok(_) => {
            if *passed {
                interaction.reply("Submitted trial").await?;
            } else {
                interaction
                    .reply("Submitted trial, the notes have been sent to the trainee")
                    .await?;
            }
        }
    };
    Ok(())
}

async fn trial_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name != "certification" {
        return Ok(());
    }
    let Ok(Ok((Response::ListInstructor(Ok(certs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
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
        .filter(|c| c.name.to_lowercase().contains(&focus.value.to_lowercase()))
        .collect();
    if certs.len() > 25 {
        certs.truncate(25);
    }
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for cert in certs {
                f = f.add_string_choice(&cert.name, cert.id);
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn view(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching certifications...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    let Ok(Ok((Response::Active(Ok(certs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Active { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch certifications").await;
    };
    if certs.is_empty() {
        return interaction
            .reply(format!("<@{user}> has no certifications"))
            .await;
    }
    let mut content = format!("**<@{user}> Certifications**\n\n");
    for cert in certs {
        if let Ok(Ok((Response::Name(Ok(Some(name))), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::certifications::db,
            Name {
                certification: cert.certification
            }
        )
        .await
        {
            content.push_str(&format!(
                "**{}**\nSince: <t:{}:D>\n{}\n\n",
                name,
                cert.created.unix_timestamp(),
                if let Some(valid_for) = cert.valid_for {
                    format!(
                        "Until: <t:{}:D> ({} days)",
                        cert.valid_until
                            .expect("should have been generated by postgres")
                            .unix_timestamp(),
                        valid_for
                    )
                } else {
                    "No expiration".to_string()
                }
            ));
        }
    }
    interaction.reply(content).await
}

async fn list(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
    available: bool,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching certifications...").await?;
    let Ok(Ok((Response::List(Ok(mut certs)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        List {}
    )
    .await
    else {
        return interaction.reply("Failed to fetch certifications").await;
    };
    if available {
        let member = get_option_user!(options, "member").map_or_else(
            || {
                command
                    .member
                    .as_ref()
                    .expect("member should always exist on guild commands")
                    .user
                    .id
            },
            |user| *user,
        );
        let mut member_roles = GUILD.member(&ctx.http, member).await.map_or_else(
            |e| {
                error!("Failed to fetch member: {}", e);
                Vec::new()
            },
            |m| m.roles,
        );
        if member_roles.contains(&MEMBER) {
            member_roles.push(JUNIOR);
        }
        certs.retain(|cert| {
            (&cert.roles_required.iter().cloned().collect::<HashSet<_>>()
                - &member_roles
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<HashSet<_>>())
                .is_empty()
        });
        certs.retain(|cert| {
            !(&cert.roles_granted.iter().cloned().collect::<HashSet<_>>()
                - &member_roles
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<HashSet<_>>())
                .is_empty()
        });
    }
    if certs.is_empty() {
        return interaction.reply("There are no certifications").await;
    }
    let mut content = String::new();
    for cert in certs {
        content.push_str(&format!(
            "**{}**\n<{}>\n{}\n{}\n{}\n\n",
            cert.name,
            cert.link,
            {
                if available {
                    cert.instructors.map_or_else(
                        || "No instructors".to_string(),
                        |ins| {
                            let instructors =
                                ins.iter().map(|i| format!("<@{i}>")).collect::<Vec<_>>();
                            if instructors.is_empty() {
                                "No instructors".to_string()
                            } else {
                                format!("Instructor(s): {}", instructors.join(", "))
                            }
                        },
                    )
                } else {
                    String::new()
                }
            },
            {
                let requires = cert
                    .roles_required
                    .iter()
                    .map(|r| format!("<@&{r}>"))
                    .collect::<Vec<_>>()
                    .join(", ");
                if requires.is_empty() {
                    "No requirements".to_string()
                } else {
                    format!("Requires: {requires}")
                }
            },
            cert.valid_for.map_or_else(
                || "No expiration".to_string(),
                |v| format!("Valid for {v} days")
            )
        ));
    }
    interaction.reply(content).await
}
