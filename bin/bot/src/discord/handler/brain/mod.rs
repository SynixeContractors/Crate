mod functions;

use std::{collections::HashMap, sync::atomic::AtomicBool};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs, ReasoningEffort,
    },
};
use serde_json::json;
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
                Box::new(functions::bank::Spending {}),
                Box::new(functions::bank::History {}),
                Box::new(functions::bank::ShopPrice {}),
                Box::new(functions::missions::Upcoming {}),
                Box::new(functions::moderation::Timeout {}),
            ],
        }
    }

    pub fn awake(&self) -> bool {
        self.awake.load(std::sync::atomic::Ordering::Relaxed)
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::significant_drop_tightening)] // false positive on conversations
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
            let messages = self
                .conversations
                .read()
                .await
                .get(&message.channel_id)
                .cloned()
                .unwrap_or_default();
            let mut args = CreateChatCompletionRequestArgs::default();
            args.max_completion_tokens(512u32)
                .model("gpt-6-luna")
                .reasoning_effort(ReasoningEffort::None)
                .messages(messages);
            if max_iter != 1 {
                args.tools(
                    self.functions
                        .iter()
                        .map(|f| f.to_openai())
                        .collect::<Vec<_>>(),
                );
            }
            let Ok(request) = args.build() else {
                return None;
            };
            match self.client.chat().create(request).await {
                Ok(response) => {
                    let response = response.choices.first()?.message.clone();
                    if let Some(tool_calls) = response.tool_calls.filter(|calls| !calls.is_empty())
                    {
                        let mut outputs = Vec::with_capacity(tool_calls.len());
                        for call in &tool_calls {
                            outputs.push(match call {
                                ChatCompletionMessageToolCalls::Function(call) => (
                                    call.id.clone(),
                                    self.run_function(
                                        ctx,
                                        &call.function.name,
                                        &call.function.arguments,
                                    )
                                    .await,
                                ),
                                ChatCompletionMessageToolCalls::Custom(call) => (
                                    call.id.clone(),
                                    json!({"error": "custom tool calls are not supported"})
                                        .to_string(),
                                ),
                            });
                        }
                        let mut conversations = self.conversations.write().await;
                        let conversation = conversations.entry(message.channel_id).or_default();
                        conversation.push(
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .tool_calls(tool_calls)
                                .build()
                                .expect("prompt is valid")
                                .into(),
                        );
                        for (id, output) in outputs {
                            conversation.push(
                                ChatCompletionRequestToolMessageArgs::default()
                                    .tool_call_id(id)
                                    .content(output)
                                    .build()
                                    .expect("prompt is valid")
                                    .into(),
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

    async fn run_function(&self, ctx: &Context, name: &str, arguments: &str) -> String {
        println!("ask function_call: {name} {arguments}");
        let Some(function) = self.functions.iter().find(|f| f.name() == name) else {
            return json!({"error": "function not found"}).to_string();
        };
        let arguments = serde_json::from_str(arguments).unwrap_or(serde_json::Value::Null);
        function.run(ctx, arguments).await.map_or_else(
            || json!({"error": "function failed as called"}).to_string(),
            |response| {
                println!("ask response: {response}");
                response.to_string()
            },
        )
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
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(Self::prompt(message.channel_id))
                    .build()
                    .expect("prompt is valid")
                    .into(),
            );
        }
        if message.author.bot {
            if let Ok(chat) = ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()
            {
                conversation.push(chat.into());
            }
        } else if let Ok(chat) = ChatCompletionRequestUserMessageArgs::default()
            .content(content)
            .build()
        {
            conversation.push(chat.into());
        }
        loop {
            let total_length = conversation
                .iter()
                .map(|c| serde_json::to_string(c).map_or(0, |s| s.len()))
                .sum::<usize>();
            if total_length > 8192 {
                conversation.remove(1);
                while matches!(
                    conversation.get(1),
                    Some(ChatCompletionRequestMessage::Tool(_))
                ) {
                    conversation.remove(1);
                }
                println!("remove old message");
            } else {
                break;
            }
        }
    }

    fn prompt(channel: ChannelId) -> String {
        let mut prompt = include_str!("prompt.txt").to_string();
        prompt = prompt.replace("%channel%", &format!("<#{}>", channel));
        prompt
    }
}
