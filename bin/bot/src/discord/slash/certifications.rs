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

use crate::discord::interaction::{Generic, Interaction};

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

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "trial" => trial(ctx, command, &subcommand.options).await,
            "view" => view(ctx, command, &subcommand.options).await,
            "list" => list(ctx, command, &subcommand.options, false).await,
            "available" => list(ctx, command, &subcommand.options, true).await,
            _ => unreachable!(),
        }
    }
}

pub async fn autocomplete(ctx: &Context, autocomplete: &AutocompleteInteraction) {
    let subcommand = autocomplete.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand && subcommand.name.as_str() == "trial" {
        trial_autocomplete(ctx, autocomplete, &subcommand.options).await;
    }
}

async fn trial(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    interaction.reply("Fetching certifications...").await;
    let Ok(((Response::ListInstructor(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
            member: command.member.as_ref().unwrap().user.id
        }
    )
    .await else {
        interaction.reply("Failed to fetch certifications").await;
        return;
    };
    if certs.is_empty() {
        interaction
            .reply("You are not an instructor for any certifications")
            .await;
        return;
    }
    let cert = options
        .iter()
        .find(|option| option.name == "certification")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();
    let Some(cert) = certs.iter().find(|c| c.name == cert) else {
        interaction.reply("Invalid certification").await;
        return;
    };
    let passed = options
        .iter()
        .find(|option| option.name == "passed")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_bool()
        .unwrap();
    let CommandDataOptionValue::User(trainee, _member) = options
        .iter()
        .find(|option| option.name == "trainee")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid trainee");
    };
    let notes = options
        .iter()
        .find(|option| option.name == "notes")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    interaction.reply("Submitting trial...").await;
    match events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Certify {
            instructor: command.member.as_ref().unwrap().user.id,
            trainee: trainee.id,
            certification: cert.id,
            notes: notes.clone(),
            passed,
        }
    )
    .await
    {
        Err(e) | Ok(((Response::Certify(Err(e)), _), _)) => {
            interaction
                .reply(format!("Failed to submit trial: {e}"))
                .await;
        }
        Ok(_) => {
            if passed {
                interaction.reply("Submitted trial").await;
            } else {
                interaction
                    .reply("Submitted trial, the notes have been sent to the trainee")
                    .await;
            }
        }
    };
}

async fn trial_autocomplete(
    ctx: &Context,
    autocomplete: &AutocompleteInteraction,
    options: &[CommandDataOption],
) {
    let focus = options.iter().find(|o| o.focused);
    let Some(focus) = focus else {
        return;
    };
    if focus.name != "certification" {
        return;
    }
    let Ok(((Response::ListInstructor(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
            member: autocomplete.user.id
        }
    )
    .await else {
        error!("Failed to fetch certifications");
        return;
    };
    let mut certs: Vec<_> = certs
        .into_iter()
        .filter(|c| {
            c.name.to_lowercase().contains(
                &focus
                    .value
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
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
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    interaction.reply("Fetching certifications...").await;
    let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid member");
    };
    let Ok(((Response::Active(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Active { member: user.id }
    )
    .await
    else {
        interaction.reply("Failed to fetch certifications").await;
        return;
    };
    if certs.is_empty() {
        interaction
            .reply(format!("<@{}> has no certifications", user.id))
            .await;
        return;
    }
    let mut content = format!("**<@{}> Certifications**\n\n", user.id);
    for cert in certs {
        if let Ok(((Response::Name(Ok(Some(name))), _), _)) = events_request!(
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
                        cert.valid_until.unwrap().unix_timestamp(),
                        valid_for
                    )
                } else {
                    "No expiration".to_string()
                }
            ));
        }
    }
    interaction.reply(content).await;
}

async fn list(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
    available: bool,
) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    interaction.reply("Fetching certifications...").await;
    let Ok(((Response::List(Ok(mut certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        List {}
    )
    .await else {
        interaction.reply("Failed to fetch certifications").await;
        return;
    };
    if available {
        let mut member_roles = command.member.as_ref().unwrap().roles.clone();
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
        interaction.reply("There are no certifications").await;
        return;
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
    interaction.reply(content).await;
}
