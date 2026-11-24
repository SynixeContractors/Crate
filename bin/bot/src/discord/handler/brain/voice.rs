//! Brodsky's reactions to things that happen, and the "right now" context he's given in chat

use std::{fmt::Write, sync::LazyLock};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ReasoningEffort,
    },
};
use synixe_events::missions::db::Response;
use synixe_proc::events_request_5;

use super::{MODEL, mood};

static CLIENT: LazyLock<Client<OpenAIConfig>> = LazyLock::new(Client::new);

/// Who Brodsky is, shared by chat and reactions
pub const PERSONA: &str = include_str!("persona.txt");

/// What's going on right now: the date, his mood and why, and the current or next mission
pub async fn right_now() -> String {
    let mut now = format!("It is {}.\n", time::OffsetDateTime::now_utc().date());
    if let Some(report) = mood::current().await {
        let _ = writeln!(now, "{} {}", report.summary(), report.mood.behaviour());
    }
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FetchCurrentMission {}
    )
    .await
    {
        Ok(Ok((Response::FetchCurrentMission(Ok(Some(current))), _))) => {
            let _ = writeln!(
                now,
                "The mission \"{}\" is being played right now: {}",
                current.name, current.summary
            );
        }
        _ => {
            if let Ok(Ok((Response::UpcomingSchedule(Ok(mut upcoming)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::missions::db,
                UpcomingSchedule {}
            )
            .await
            {
                upcoming.sort_by_key(|scheduled| scheduled.start);
                if let Some(next) = upcoming.first() {
                    let _ = writeln!(
                        now,
                        "The next mission is \"{}\" at <t:{}:F>: {}",
                        next.name,
                        next.start.unix_timestamp(),
                        next.summary
                    );
                }
            }
        }
    }
    now
}

/// The most words in a reaction
const REACTION_WORDS: usize = 30;

/// Brodsky's reaction to something that happened, as one short line. `None` if he can't be reached,
/// so the caller can stay quiet or fall back to a canned line.
pub async fn react(situation: &str) -> Option<String> {
    let mood = mood::current().await.map_or_else(String::new, |report| {
        format!("{} {}", report.summary(), report.mood.behaviour())
    });
    let request = CreateChatCompletionRequestArgs::default()
        .model(MODEL)
        .reasoning_effort(ReasoningEffort::None)
        .max_completion_tokens(120u32)
        .messages(vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(format!(
                    "{PERSONA}\n{mood}\n\nReact to what just happened, in character, as one short line of at most {REACTION_WORDS} words. Only the line itself: no quotes, no name, no hashtags, no emoji spam."
                ))
                .build()
                .ok()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(situation)
                .build()
                .ok()?
                .into(),
        ])
        .build()
        .ok()?;
    match CLIENT.chat().create(request).await {
        Ok(response) => {
            let line = response.choices.first()?.message.content.clone()?;
            let line = line.trim().trim_matches('"').trim();
            (!line.is_empty()).then(|| line.to_string())
        }
        Err(e) => {
            error!("brodsky failed to react: {e:?}");
            None
        }
    }
}
