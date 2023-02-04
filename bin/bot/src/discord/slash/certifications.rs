use std::collections::HashSet;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
        autocomplete::AutocompleteInteraction,
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::certifications::db::Response;
use synixe_meta::discord::role::{JUNIOR, MEMBER};
use synixe_proc::events_request;

use crate::{
    discord::interaction::{Generic, Interaction},
    get_option, get_option_user,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("certifications")
        .description("Certifications")
        .create_option(|option| {
            option
                .name("trial")
                .description("You ran someone through a certification trial")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("trainee")
                        .description("The person you ran through the trial")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("certification")
                        .description("The certification you ran them through")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("passed")
                        .description("Did the trainee pass the trial?")
                        .kind(CommandOptionType::Boolean)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("notes")
                        .description(
                            "Notes about the trial, only shared between you and the trainee",
                        )
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("view")
                .description("View someone's certifications")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to view certifications for")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("list")
                .description("List all certifications")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("available")
                .description("List all certifications available to you")
                .kind(CommandOptionType::SubCommand)
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "trial" => trial(ctx, command, &subcommand.options).await?,
            "view" => view(ctx, command, &subcommand.options).await?,
            "list" => list(ctx, command, &subcommand.options, false).await?,
            "available" => list(ctx, command, &subcommand.options, true).await?,
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
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "trial" {
        trial_autocomplete(ctx, autocomplete, &subcommand.options).await?;
    }
    Ok(())
}

async fn trial(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Fetching certifications...").await?;
    let Ok(Ok((Response::ListInstructor(Ok(certs)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
            member: command.member.as_ref().expect("member should always exist on guild commands").user.id
        }
    )
    .await else {
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
    match events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Certify {
            instructor: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id,
            trainee: trainee.id,
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
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let focus = options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return Ok(());
    };
    if focus.name != "certification" {
        return Ok(());
    }
    let Ok(Ok((Response::ListInstructor(Ok(certs)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
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

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Fetching certifications...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    let Ok(Ok((Response::Active(Ok(certs)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Active { member: user.id }
    )
    .await
    else {
        return interaction.reply("Failed to fetch certifications").await;
    };
    if certs.is_empty() {
        return interaction
            .reply(format!("<@{}> has no certifications", user.id))
            .await;
    }
    let mut content = format!("**<@{}> Certifications**\n\n", user.id);
    for cert in certs {
        if let Ok(Ok((Response::Name(Ok(Some(name))), _))) = events_request!(
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
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
    available: bool,
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Application(command), options);
    interaction.reply("Fetching certifications...").await?;
    let Ok(Ok((Response::List(Ok(mut certs)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        List {}
    )
    .await else {
        return interaction.reply("Failed to fetch certifications").await;
    };
    if available {
        let mut member_roles = command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles
            .clone();
        if member_roles.contains(&MEMBER) {
            member_roles.push(JUNIOR);
        }
        certs.retain(|cert| {
            (&cert.roles_required.iter().cloned().collect::<HashSet<_>>()
                - &member_roles
                    .iter()
                    .map(|r| r.0.to_string())
                    .collect::<HashSet<_>>())
                .is_empty()
        });
        certs.retain(|cert| {
            !(&cert.roles_granted.iter().cloned().collect::<HashSet<_>>()
                - &member_roles
                    .iter()
                    .map(|r| r.0.to_string())
                    .collect::<HashSet<_>>())
                .is_empty()
        });
    }
    if certs.is_empty() {
        return interaction.reply("There are no certifications").await;
    }
    let mut content = "**Certifications**\n\n".to_string();
    for cert in certs {
        content.push_str(&format!(
            "**{}**\n<{}>\n{}\n{}\n\n",
            cert.name,
            cert.link,
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
