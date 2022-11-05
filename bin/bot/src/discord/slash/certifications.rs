use std::time::Duration;

use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
        command::CommandOptionType,
        InteractionResponseType,
    },
    prelude::Context,
};
use synixe_events::certifications::db::Response;
use synixe_proc::events_request;

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
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "trial" => trial(ctx, command, &subcommand.options).await,
            "view" => view(ctx, command, &subcommand.options).await,
            "list" => list(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn trial(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    command
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content("Fetching certifications...").ephemeral(true)
                })
        })
        .await
        .unwrap();
    if let Ok(((Response::ListInstructor(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        ListInstructor {
            member: command.member.as_ref().unwrap().user.id
        }
    )
    .await
    {
        if certs.is_empty() {
            command
                .edit_original_interaction_response(&ctx, |m| {
                    m.content("You are not an instructor for any certifications")
                })
                .await
                .unwrap();
            return;
        }
        let m = command
            .create_followup_message(&ctx, |r| {
                r.content("Select certification")
                    .ephemeral(true)
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_select_menu(|m| {
                                m.custom_id("cert").options(|o| {
                                    for cert in &certs {
                                        o.create_option(|o| {
                                            o.label(cert.name.clone()).value(cert.id.to_string())
                                        });
                                    }
                                    o
                                })
                            })
                        })
                    })
            })
            .await
            .unwrap();
        if let Some(interaction) = m
            .await_component_interaction(ctx)
            .timeout(Duration::from_secs(60 * 3))
            .collect_limit(1)
            .await
        {
            let cert_id = interaction.data.values.first().unwrap();
            let passed = options
                .iter()
                .find(|option| option.name == "passed")
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .as_bool()
                .unwrap();
            let trainee = if let CommandDataOptionValue::User(user, _member) = options
                .iter()
                .find(|option| option.name == "trainee")
                .unwrap()
                .resolved
                .as_ref()
                .unwrap()
            {
                user
            } else {
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
            command
                .edit_followup_message(&ctx, m.id, |r| r.content(cert_id).components(|c| c))
                .await
                .unwrap();
            match events_request!(
                bootstrap::NC::get().await,
                synixe_events::certifications::db,
                Certify {
                    instructor: command.member.as_ref().unwrap().user.id,
                    trainee: trainee.id,
                    certification: cert_id.parse().unwrap(),
                    notes: notes.clone(),
                    passed,
                }
            )
            .await
            {
                Err(e) | Ok(((Response::Certify(Err(e)), _), _)) => {
                    command
                        .edit_followup_message(&ctx, m.id, |r| {
                            r.content(format!("Failed to log trial: {}", e))
                        })
                        .await
                        .unwrap();
                }
                Ok(_) => {
                    if passed {
                        command
                            .edit_followup_message(&ctx, m.id, |r| r.content("Trial logged"))
                            .await
                            .unwrap();
                    } else {
                        command
                            .edit_followup_message(&ctx, m.id, |r| {
                                r.content("Trial logged, the notes have been sent to the trainee")
                            })
                            .await
                            .unwrap();
                    }
                }
            };
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    command
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content("Fetching certifications...").ephemeral(true)
                })
        })
        .await
        .unwrap();
    let member = if let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
    {
        user.id
    } else {
        panic!("Invalid member");
    };
    if let Ok(((Response::Active(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        Active { member }
    )
    .await
    {
        if certs.is_empty() {
            command
                .edit_original_interaction_response(&ctx, |m| {
                    m.content(format!("<@{}> has no certifications", member,))
                })
                .await
                .unwrap();
            return;
        }
        let mut content = format!("**<@{}> Certifications**\n\n", member);
        for cert in certs {
            if let Ok(((Response::Name(Ok(name)), _), _)) = events_request!(
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
        command
            .create_followup_message(ctx, |m| m.content(content).ephemeral(true))
            .await
            .unwrap();
    } else {
        command
            .edit_original_interaction_response(&ctx, |m| {
                m.content("Failed to fetch certifications")
            })
            .await
            .unwrap();
    }
}

#[allow(clippy::too_many_lines)]
async fn list(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[CommandDataOption],
) {
    command
        .create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                .interaction_response_data(|m| {
                    m.content("Fetching certifications...").ephemeral(true)
                })
        })
        .await
        .unwrap();
    if let Ok(((Response::List(Ok(certs)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        List {}
    )
    .await
    {
        if certs.is_empty() {
            command
                .edit_original_interaction_response(&ctx, |m| {
                    m.content("There are no certifications")
                })
                .await
                .unwrap();
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
                        .map(|r| format!("<@&{}>", r))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if requires.is_empty() {
                        "No requirements".to_string()
                    } else {
                        format!("Requires: {}", requires)
                    }
                },
                cert.valid_for.map_or_else(
                    || "No expiration".to_string(),
                    |v| format!("Valid for {} days", v)
                )
            ));
        }
        command
            .create_followup_message(ctx, |m| m.content(content).ephemeral(true))
            .await
            .unwrap();
    } else {
        command
            .edit_original_interaction_response(&ctx, |m| {
                m.content("Failed to fetch certifications")
            })
            .await
            .unwrap();
    }
}
