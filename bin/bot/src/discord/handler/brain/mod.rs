mod functions;

use std::{collections::HashMap, sync::atomic::AtomicBool};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
};
use serenity::{
    model::prelude::{ChannelId, Message},
    prelude::Context,
};
use synixe_meta::discord::GUILD;
use tokio::sync::RwLock;

use self::functions::BrainFunction;

pub struct Brain {
    awake: AtomicBool,
    client: Client<OpenAIConfig>,
    conversations: RwLock<HashMap<ChannelId, Vec<ChatCompletionRequestMessage>>>,
    functions: Vec<Box<dyn BrainFunction>>,
}

impl Brain {
    pub fn new() -> Self {
        Self {
            awake: AtomicBool::new(true),
            client: Client::new(),
            conversations: RwLock::new(HashMap::new()),
            functions: vec![
                Box::new(functions::bank::GetBalance {}),
                Box::new(functions::moderation::Timeout {}),
            ],
        }
    }

    pub fn awake(&self) -> bool {
        self.awake.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn ask(&self, ctx: &Context, message: &Message) -> Option<String> {
        self.observe(ctx, message).await;
        let mut max_iter = 10;
        loop {
            max_iter -= 1;
            if max_iter == 0 {
                return Some(String::from(
                    "I'm sorry, my head is spinning. I can't think of what to say.",
                ));
            }
            let Ok(request) = CreateChatCompletionRequestArgs::default()
                .max_tokens(512u16)
                .model("gpt-4o-mini")
                .messages(
                    self.conversations
                        .read()
                        .await
                        .get(&message.channel_id)
                        .cloned()
                        .unwrap_or_default(),
                )
                .functions(if max_iter == 1 {
                    Vec::new()
                } else {
                    self.functions
                        .iter()
                        .map(|f| f.to_openai())
                        .collect::<Vec<_>>()
                })
                .build()
            else {
                return None;
            };
            match self.client.chat().create(request).await {
                Ok(response) => {
                    let response = response.choices.first()?.message.clone();
                    if let Some(function_call) = response.function_call {
                        println!("ask function_call: {function_call:?}");
                        let Some(function) = self
                            .functions
                            .iter()
                            .find(|f| f.name() == function_call.name)
                        else {
                            self.conversations
                                .write()
                                .await
                                .entry(message.channel_id)
                                .or_default()
                                .push(
                                    ChatCompletionRequestMessageArgs::default()
                                        .role(Role::Function)
                                        .name(function_call.name)
                                        .content("{\"error\": \"function not found\"}")
                                        .build()
                                        .expect("prompt is valid"),
                                );
                            continue;
                        };
                        let Some(response) = function
                            .run(
                                ctx,
                                function_call
                                    .arguments
                                    .parse()
                                    .expect("chatgpt returns valid arguments"),
                            )
                            .await
                        else {
                            self.conversations
                                .write()
                                .await
                                .entry(message.channel_id)
                                .or_default()
                                .push(
                                    ChatCompletionRequestMessageArgs::default()
                                        .role(Role::Function)
                                        .name(function_call.name)
                                        .content("{\"error\": \"function failed as called\"}")
                                        .build()
                                        .expect("prompt is valid"),
                                );
                            continue;
                        };
                        println!("ask response: {response}");
                        if let Some(conversation) = self
                            .conversations
                            .write()
                            .await
                            .get_mut(&message.channel_id)
                        {
                            conversation.push(
                                ChatCompletionRequestMessageArgs::default()
                                    .role(Role::Function)
                                    .name(function.name())
                                    .content(response.to_string())
                                    .build()
                                    .expect("prompt is valid"),
                            );
                        }
                    } else if let Some(content) = response.content {
                        println!("ask content: {content}");
                        return Some(if content.contains("| Ctirad Brodsky: ") {
                            content.split_once("| Ctirad Brodsky: ")?.1.to_string()
                        } else {
                            content
                        });
                    }
                }
                Err(e) => {
                    error!("ask error: {:?}", e);
                    return None;
                }
            }
        }
    }

    #[allow(clippy::significant_drop_tightening)] // false positive on conversations
    pub async fn observe(&self, ctx: &Context, message: &Message) {
        let mut name = message.author.name.clone();
        if let Ok(author) = GUILD.member(&ctx, message.author.id).await
            && let Some(nick) = &author.nick
        {
            name.clone_from(nick);
        }
        let mut content = format!("|{}| {}: {}", message.timestamp, name, message.content);
        for user in &message.mentions {
            if let Ok(member) = GUILD.member(&ctx, user.id).await
                && let Some(nick) = &member.nick
            {
                content = content.replace(&format!("<@{}>", user.id), nick);
            }
        }
        content = content.replace("<@1028418063168708638>", "Ctirad Brodsky");
        println!("observe content: {content}");
        let mut conversations = self.conversations.write().await;
        let conversation = conversations.entry(message.channel_id).or_default();
        if conversation.is_empty() {
            conversation.push(
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(Self::prompt(message.channel_id))
                    .build()
                    .expect("prompt is valid"),
            );
        }
        if let Ok(chat) = ChatCompletionRequestMessageArgs::default()
            .role(if message.author.bot {
                Role::Assistant
            } else {
                Role::User
            })
            .content(content)
            .build()
        {
            conversation.push(chat);
        }
        loop {
            let total_length = conversation
                .iter()
                .map(|c| c.content.clone().unwrap_or_default().len())
                .sum::<usize>();
            if total_length > 8192 {
                conversation.remove(1);
                println!("remove old message");
            } else {
                break;
            }
        }
    }

    fn prompt(channel: ChannelId) -> String {
        let mut prompt = include_str!("prompt.txt").to_string();
        prompt = prompt.replace("%channel%", &format!("<#{}>", &channel));
        prompt
    }
}
