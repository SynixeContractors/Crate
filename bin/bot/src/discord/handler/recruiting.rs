use chatgpt::{
    prelude::{ChatGPT, ModelConfiguration},
    types::{ChatMessage, Role},
};
use serenity::{model::prelude::Message, prelude::Context};

pub async fn get_reply(message: String) -> Option<String> {
    let key = std::env::var("OPENAI_KEY").ok()?;
    let client = ChatGPT::new_with_config(
        key,
        ModelConfiguration {
            temperature: 0.7,
            ..Default::default()
        },
    )
    .expect("failed to create chatgpt client");
    match client
        .send_history(&{
            vec![
                ChatMessage {
                    role: Role::System,
                    content: include_str!("recruiting-prompt.txt").to_string(),
                },
                ChatMessage {
                    role: Role::User,
                    content: message,
                },
            ]
        })
        .await
    {
        Ok(response) => {
            let response = response.message_choices[0].message.clone().content;
            if response.is_empty() || response.starts_with("Denied") {
                None
            } else {
                let response = response.trim_start_matches("Accepted. ");
                Some(response.to_string())
            }
        }
        Err(e) => {
            error!("failed to send history: {:?}", e);
            None
        }
    }
}

pub async fn check_embed(ctx: &Context, message: &Message) -> Option<String> {
    if !message.content.is_empty() {
        return None;
    }
    if let Some(data) = message.embeds.first() {
        let content = format!(
            "{}\n\n{}",
            data.title.as_ref().expect("embed has title"),
            data.description.as_ref().expect("embed has description")
        );
        if let Some(reply) = get_reply(content).await {
            message.reply_ping(ctx, format!("I'm not replying yet, just in testing. I do think this is a good candidate though, this is what I would've sent.\n\n> {reply}")).await;
            return Some(reply);
        }
    }
    None
}
