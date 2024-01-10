use async_openai::types::{ChatCompletionFunctions, ChatCompletionFunctionsArgs};
use serenity::prelude::Context;

pub mod bank;
pub mod moderation;

#[async_trait::async_trait]
pub trait BrainFunction: 'static + Send + Sync {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn args(&self) -> serde_json::Value;
    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value>;

    fn to_openai(&self) -> ChatCompletionFunctions {
        ChatCompletionFunctionsArgs::default()
            .name(self.name())
            .description(self.desc())
            .parameters(self.args())
            .build()
            .expect("failed to build ChatCompletionFunctionsArgs")
    }
}
