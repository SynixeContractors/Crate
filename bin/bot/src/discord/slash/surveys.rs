use serenity::all::{
    ButtonStyle, CommandData, CommandDataOption, CommandDataOptionValue, CommandInteraction,
    CommandOptionType, ComponentInteraction, Context, CreateActionRow, CreateAutocompleteResponse,
    CreateButton, CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
    CreateMessage,
};
use synixe_events::surveys::db::Response;
use synixe_meta::discord::role::STAFF;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::{discord::interaction::Interaction, get_option};

use super::ShouldAsk;

pub fn register() -> CreateCommand {
    CreateCommand::new("surveys")
        .description("surveys")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "post", "Post a Survey")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "survey",
                        "Select the survey to post",
                    )
                    .required(true)
                    .set_autocomplete(true),
                ),
        )
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
            "post" => post_autocomplete(ctx, autocomplete).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for missions provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "post" => post(ctx, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn post_autocomplete(
    ctx: &Context,
    autocomplete: &CommandInteraction,
) -> serenity::Result<()> {
    let Some(focus) = CommandData::autocomplete(&autocomplete.data) else {
        return Ok(());
    };
    if focus.name != "survey" {
        return Ok(());
    }
    let Ok(Ok((Response::SearchSurvey(Ok(mut surveys)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::surveys::db,
        SearchSurvey {
            title: if focus.value.is_empty() {
                None
            } else {
                Some(focus.value.to_string())
            },
        }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    surveys.truncate(25);
    if let Err(e) = autocomplete
        .create_response(&ctx.http, {
            let mut f = CreateAutocompleteResponse::default();
            for (id, title) in surveys {
                f = f.add_string_choice(title, id);
            }
            CreateInteractionResponse::Autocomplete(f)
        })
        .await
    {
        error!("failed to create autocomplete response: {}", e);
    }
    Ok(())
}

async fn post(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Some(channel) = &command.channel else {
        return interaction
            .reply("This command must be run in a guild channel")
            .await;
    };
    super::requires_roles(
        command.user.id,
        &[STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;
    let Some(survey) = get_option!(options, "survey", String) else {
        return interaction
            .reply("Required option not provided: survey")
            .await;
    };
    let survey: Uuid = survey.parse().expect("survey id should be a valid uuid");
    let Ok(Ok((Response::GetSurvey(Ok(Some((id, title, description)))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::surveys::db,
        GetSurvey { survey }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    if let Err(e) = channel
        .id
        .send_message(
            &ctx,
            CreateMessage::default()
                .add_embed(CreateEmbed::default().title(title).description(description))
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!("survey_submit:{id}"))
                        .style(ButtonStyle::Primary)
                        .label("Submit"),
                ])]),
        )
        .await
    {
        error!("failed to send survey message: {}", e);
    }
    interaction.reply("Survey posted").await?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub async fn submit_button(
    ctx: &Context,
    component: &ComponentInteraction,
) -> serenity::Result<()> {
    let survey = component
        .data
        .custom_id
        .strip_prefix("survey_submit:")
        .expect("custom id should start with survey_submit:")
        .parse()
        .expect("survey id should be a valid uuid");
    let mut interaction = Interaction::new(ctx, component.clone(), &[]);
    let Ok(Ok((Response::GetOptions(Ok(options)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::surveys::db,
        GetOptions { survey }
    )
    .await
    else {
        error!("failed to fetch mission list");
        return Ok(());
    };
    let Some(option) = interaction
        .choice(
            "Please select an option",
            &options
                .iter()
                .map(|option| (option.clone(), option.clone()))
                .collect::<Vec<_>>(),
        )
        .await?
    else {
        warn!("No option selected");
        return Ok(());
    };
    let Ok(Ok((Response::Submit(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::surveys::db,
        Submit {
            survey,
            member: component.user.id,
            option
        }
    )
    .await
    else {
        error!("failed to submit survey");
        return Ok(());
    };
    interaction.reply("Survey submitted!").await?;
    Ok(())
}
