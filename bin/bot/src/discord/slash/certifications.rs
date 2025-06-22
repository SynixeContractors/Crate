use std::fmt::Write;
use std::{collections::HashSet, time::Duration};

use serenity::all::{ComponentInteractionDataKind, CreateInteractionResponseMessage};
use serenity::{
    all::{
        CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
        CommandOptionType, CreateActionRow, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption,
    },
    builder::{
        CreateAutocompleteResponse, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    },
    client::Context,
};
use synixe_events::certifications::db::Response;
use synixe_meta::discord::{
    GUILD,
    role::{JUNIOR, MEMBER},
};
use synixe_model::certifications::Certification;
use synixe_proc::{events_request_2, events_request_5};

use crate::{
    discord::interaction::{Confirmation, Interaction},
    get_option, get_option_user,
};

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

#[allow(clippy::too_many_lines)]
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
    let trainee_roles = GUILD.member(&ctx.http, *trainee).await.map_or_else(
        |e| {
            error!("Failed to fetch trainee: {}", e);
            Vec::new()
        },
        |m| m.roles,
    );
    let missing = cert
        .roles_required
        .iter()
        .filter(|r| !trainee_roles.iter().any(|tr| &tr.to_string() == *r))
        .collect::<Vec<_>>();
    if !missing.is_empty() && interaction.confirm(
            &format!(
                "The trainee is missing the following roles: {}\nDo you want to submit the trial anyway?",
                missing.iter().map(|r| format!("<@&{r}>")).collect::<Vec<_>>().join(", ")
            )
        ).await? != Confirmation::Yes {
        interaction.reply("Trial not submitted").await?;
        return Ok(());
    }
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
            passed: *passed,
            notes: "Discord command".to_string(),
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
            interaction.reply("Submitted trial").await?;
        }
    }

    if *passed {
        if let Err(e) = process_trial_first_kit(ctx, interaction, cert, trainee).await {
            error!("Failed to process first kit: {}", e);
        }
    }
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
    certs.truncate(25);
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
            write!(
                content,
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
            )?;
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
        write!(
            content,
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
        )?;
    }
    interaction.reply(content).await
}

#[allow(clippy::too_many_lines)]
async fn process_trial_first_kit(
    ctx: &Context,
    mut interaction: Interaction<'_>,
    cert: &Certification,
    trainee: &serenity::all::UserId,
) -> serenity::Result<()> {
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        PassedCount {
            certification: cert.id,
            member: *trainee,
        }
    )
    .await
    {
        Err(e) => {
            error!("Failed to check passed count: {}", e);
            if let Err(e) = synixe_meta::discord::channel::LOG
                .say(
                    ctx,
                    format!(
                        "When submitting a trial for <@{}> in {}, the passed count could not be checked: {}",
                        trainee,
                        cert.name,
                        e
                    ),
                )
                .await
            {
                error!("Failed to send message: {}", e);
            }
            return Ok(());
        }
        Ok(Ok((Response::PassedCount(Ok(Some(count))), _))) => {
            if count != 1 {
                return Ok(());
            }
        }
        _ => (),
    }

    // Check the number of first kits
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::certifications::db,
        FirstKits {
            certification: Some(cert.id),
        }
    )
    .await
    {
        Err(e) => {
            error!("Failed to check first kits: {}", e);
        }
        Ok(Ok((Response::FirstKits(Ok(kits)), _))) => {
            if kits.is_empty() {
                return Ok(());
            }
            if kits.len() == 1 {
                if let Err(e) = events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    GiveFirstKit {
                        first_kit: kits[0].id,
                        member: *trainee,
                    }
                )
                .await
                {
                    error!("Failed to give first kit: {}", e);
                    return interaction.reply("Failed to give first kit").await;
                }
                interaction
                    .reply("Submitted trial. First kit given")
                    .await?;
            } else {
                let Ok(dm) = trainee.create_dm_channel(ctx).await else {
                    error!("Failed to create dm channel");
                    return Ok(());
                };
                let m = dm.send_message(ctx, CreateMessage::default()
                    .content(format!("You've passed your first trial for {}! Please select a first kit to receive.", cert.name))
                    .components(vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                        "choice",
                        CreateSelectMenuKind::String {
                            options: kits
                                .iter()
                                .map(|kit| {
                                    CreateSelectMenuOption::new(kit.name.clone(), kit.id.to_string()).description(kit.description.clone().unwrap_or_default())
                                })
                                .collect::<Vec<_>>(),
                        },
                    ).custom_id("first_kit_selection"))])).await?;
                let Some(interaction) = m
                    .await_component_interaction(&ctx.shard)
                    .timeout(Duration::from_secs(60 * 60 * 24 * 7))
                    .await
                else {
                    let _ = m.reply(&ctx, "You have run out of time to select a kit, you will need to ask an admin for help.").await;
                    return Ok(());
                };

                let kit = match &interaction.data.kind {
                    ComponentInteractionDataKind::StringSelect { values } => &values[0],
                    _ => panic!("unexpected interaction data kind"),
                };

                if let Err(e) = events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    GiveFirstKit {
                        first_kit: kit.parse().expect("should be a valid first kit id"),
                        member: *trainee,
                    }
                )
                .await
                {
                    interaction
                        .create_response(
                            ctx,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::default()
                                    .content("Failed to give first kit, please contact an admin"),
                            ),
                        )
                        .await?;
                    error!("Failed to give first kit: {}", e);
                    let _ = m
                        .reply(ctx, "Failed to give first kit, please contact an admin")
                        .await;
                    if let Err(e) = synixe_meta::discord::channel::LOG
                        .say(
                            ctx,
                            format!(
                                "When <@{}> selected a first kit for {}, the first kit could not be given: {}",
                                trainee,
                                cert.name,
                                e
                            ),
                        )
                        .await
                    {
                        error!("Failed to send message: {}", e);
                    }
                    return Ok(());
                }
                interaction
                    .create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default()
                                .content("Your first kit is waiting for you in your locker!"),
                        ),
                    )
                    .await?;
            }
        }
        _ => (),
    }
    Ok(())
}
