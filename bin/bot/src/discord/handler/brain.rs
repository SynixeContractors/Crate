use std::collections::HashMap;

use chatgpt::{
    prelude::{ChatGPT, ModelConfiguration},
    types::{ChatMessage, Role},
};
use serenity::{model::prelude::ChannelId, prelude::Context};
use synixe_events::missions;
use synixe_meta::discord::channel::LOBBY;
use synixe_proc::events_request_2;
use time::{format_description, OffsetDateTime};
use tokio::sync::RwLock;

pub struct Brain {
    client: ChatGPT,
    rolling_history: RwLock<HashMap<ChannelId, (OffsetDateTime, Vec<ChatMessage>)>>,
}

impl Brain {
    pub fn new() -> Option<Self> {
        let key = std::env::var("OPENAI_KEY").ok()?;
        let client = ChatGPT::new_with_config(
            key,
            ModelConfiguration {
                temperature: 0.7,
                ..Default::default()
            },
        )
        .expect("failed to create chatgpt client");
        Some(Self {
            client,
            rolling_history: RwLock::new(HashMap::new()),
        })
    }

    pub async fn context(
        &self,
        _ctx: &Context,
        channel: ChannelId,
        message: String,
    ) -> Vec<ChatMessage> {
        let (last, mut history) = self
            .rolling_history
            .read()
            .await
            .get(&channel)
            .cloned()
            .unwrap_or_else(|| (OffsetDateTime::now_utc(), vec![]));
        if last
            .checked_add(time::Duration::minutes(30))
            .expect("can add 30 minutes")
            < OffsetDateTime::now_utc()
        {
            history = Vec::new();
        }
        if history.is_empty() {
            history = vec![ChatMessage {
                role: Role::System,
                content: create_prompt(channel).await,
            }];
            history.push(ChatMessage {
                role: Role::User,
                content: message,
            });
        } else {
            let mut last = history.last_mut().expect("last");
            last.content = format!("{}\n{}", last.content, message);
        }

        if history.len() > 20 {
            history.remove(1);
            history.remove(2);
        }
        self.rolling_history
            .write()
            .await
            .insert(channel, (OffsetDateTime::now_utc(), history.clone()));
        history
    }

    pub async fn message(
        &self,
        ctx: &Context,
        channel: ChannelId,
        message: String,
    ) -> Option<String> {
        let mut history = self.context(ctx, channel, message).await;
        match self.client.send_history(&history).await {
            Ok(resp) => {
                history.push(resp.message_choices[0].message.clone());
                self.rolling_history
                    .write()
                    .await
                    .insert(channel, (OffsetDateTime::now_utc(), history));
                Some(
                    resp.message_choices[0]
                        .message
                        .content
                        .trim_start_matches("Brodsky: ")
                        .into(),
                )
            }
            Err(e) => {
                error!("failed to send history: {:?}", e);
                None
            }
        }
    }
}

async fn create_prompt(channel: ChannelId) -> String {
    let mut start = include_str!("brain-prompt.txt").to_string();
    // Date
    start = start.replace(
        "%%date%%",
        &OffsetDateTime::now_utc()
            .format(
                &format_description::parse("[year]-[month]-[day]")
                    .expect("failed to parse date format"),
            )
            .expect("failed to format date"),
    );
    start = start.replace("%%channel%%", &format!("<#{}>", channel.0));
    start = start.replace("%%teamspeak%%", if channel == LOBBY {
        "Sorry, I can't reveal that information in a public setting, once you have joined as a recruit you'll get everything you need to know."
    } else {
        "The TeamSpeak server details are <ts.synixe.contractors>"
    });

    // Schedule
    let schedule = match events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        UpcomingSchedule {}
    )
    .await
    {
        Ok(Ok((missions::db::Response::UpcomingSchedule(Ok(upcoming)), _))) => {
            let mut content = String::from("**Upcoming Missions**\n\n");
            for scheduled in upcoming {
                content.push_str(&format!(
                    "{}\n{}\n*{}*\n\n",
                    scheduled.name,
                    scheduled
                        .start
                        .format(
                            &format_description::parse("[year]-[month]-[day] [hour]:[minute] UTC")
                                .expect("failed to parse time format")
                        )
                        .expect("failed to format time"),
                    scheduled.summary,
                ));
            }
            content
        }
        Ok(_) | Err(_) => "failed to load upcoming missions".to_string(),
    };
    start = start.replace("%%schedule%%", &schedule);

    // Next mission
    let brief = 'brief: {
        let Ok(Ok((missions::db::Response::UpcomingSchedule(Ok(missions)), _))) = events_request_2!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            UpcomingSchedule {}
        )
        .await
        else {
            break 'brief "failed to load upcoming brief".to_string();
        };
        let Some(scheduled) = missions.first() else {
            break 'brief "no upcoming brief".to_string();
        };
        scheduled
            .description
            .replace("            <br/>", "\n")
            .replace("<font color='#D81717'>", "")
            .replace("<font color='#1D69F6'>", "")
            .replace("<font color='#993399'>", "")
            .replace("<font color='#663300'>", "")
            .replace("<font color='#139120'>", "")
            .replace("</font color>", "") // felix you scoundrel
            .replace("</font>", "")
    };
    start = start.replace("%%brief%%", &brief);

    start = start.replace(
        "%%members%%",
        std::env::var("PROMPT_MEMBERS").unwrap_or_default().as_str(),
    );

    start
}
