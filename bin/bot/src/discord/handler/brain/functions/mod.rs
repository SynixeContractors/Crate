use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serenity::{all::UserId, prelude::Context};
use synixe_meta::discord::{GUILD, role::STAFF};

pub mod bank;
pub mod company;
pub mod missions;
pub mod moderation;

/// The member Brodsky is answering, which tools use to decide what they may look up or do
pub struct Caller {
    pub id: UserId,
    pub staff: bool,
}

impl Caller {
    pub async fn of(ctx: &Context, id: UserId) -> Self {
        let staff = GUILD
            .member(ctx, id)
            .await
            .is_ok_and(|member| member.roles.contains(&STAFF));
        Self { id, staff }
    }

    /// Whether they may see another member's private details, like their money
    pub fn may_see(&self, member: UserId) -> bool {
        self.staff || member == self.id
    }
}

#[async_trait::async_trait]
pub trait BrainFunction: 'static + Send + Sync {
    fn name(&self) -> &'static str;
    fn desc(&self) -> &'static str;
    fn args(&self) -> serde_json::Value;
    async fn run(
        &self,
        ctx: &Context,
        caller: &Caller,
        args: serde_json::Value,
    ) -> Option<serde_json::Value>;

    fn to_openai(&self) -> ChatCompletionTools {
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObjectArgs::default()
                .name(self.name())
                .description(self.desc())
                .parameters(self.args())
                .build()
                .expect("failed to build FunctionObject"),
        })
    }
}
