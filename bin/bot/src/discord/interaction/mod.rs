use std::{fmt::Display, time::Duration};

use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseFollowup},
    http::Http,
    model::prelude::{
        application_command::ApplicationCommandInteraction, component::ButtonStyle,
        message_component::MessageComponentInteraction, InteractionResponseType, Message,
        MessageId,
    },
    prelude::Context,
};

mod confirm;
pub use confirm::Confirmation;

pub enum Generic<'a> {
    Application(&'a ApplicationCommandInteraction),
    Message(&'a MessageComponentInteraction),
}

impl<'a> Generic<'a> {
    pub async fn create_interaction_response<'c, F>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        f: F,
    ) -> serenity::Result<()>
    where
        for<'b> F: FnOnce(&'b mut CreateInteractionResponse<'c>) -> &'b mut CreateInteractionResponse<'c>
            + Send,
    {
        match self {
            Self::Application(i) => i.create_interaction_response(http, f).await,
            Self::Message(i) => i.create_interaction_response(http, f).await,
        }
    }

    pub async fn edit_followup_message<'c, F, M: Into<MessageId> + Send>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        message_id: M,
        f: F,
    ) -> serenity::Result<Message>
    where
        for<'b> F: FnOnce(
                &'b mut CreateInteractionResponseFollowup<'c>,
            ) -> &'b mut CreateInteractionResponseFollowup<'c>
            + Send,
    {
        match self {
            Self::Application(i) => i.edit_followup_message(http, message_id, f).await,
            Self::Message(i) => i.edit_followup_message(http, message_id, f).await,
        }
    }

    pub async fn create_followup_message<'c, F>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        f: F,
    ) -> serenity::Result<Message>
    where
        for<'b> F: FnOnce(
                &'b mut CreateInteractionResponseFollowup<'c>,
            ) -> &'b mut CreateInteractionResponseFollowup<'c>
            + Send,
    {
        match self {
            Self::Application(i) => i.create_followup_message(http, f).await,
            Self::Message(i) => i.create_followup_message(http, f).await,
        }
    }
}

pub struct Interaction<'a> {
    message: Option<Message>,
    interaction: Generic<'a>,
    ctx: &'a Context,
    initial_response: bool,
}

impl<'a> Interaction<'a> {
    pub const fn new(ctx: &'a Context, interaction: Generic<'a>) -> Self {
        Self {
            message: None,
            interaction,
            ctx,
            initial_response: false,
        }
    }

    async fn initial(&mut self) {
        if !self.initial_response {
            self.interaction
                .create_interaction_response(self.ctx, |r| {
                    r.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                        .interaction_response_data(|d| d.ephemeral(true))
                })
                .await
                .unwrap();
            self.initial_response = true;
        }
    }

    pub async fn reply(&mut self, content: impl Display + Send) {
        self.initial().await;
        debug!("replying to interaction: {}", content);
        if let Some(message) = self.message.as_ref() {
            self.interaction
                .edit_followup_message(self.ctx, message.id, |m| {
                    m.content(content).components(|c| c)
                })
                .await
                .unwrap();
        } else {
            self.message = Some(
                self.interaction
                    .create_followup_message(self.ctx, |m| m.content(content).components(|c| c))
                    .await
                    .unwrap(),
            );
        }
    }

    pub async fn choice<T: ToString + Display + Sync>(
        &mut self,
        prompt: &str,
        choices: &Vec<(String, T)>,
    ) -> Option<String> {
        self.initial().await;
        debug!("prompting for choice: {}", prompt);
        if let Some(message) = self.message.as_ref() {
            self.interaction
                .edit_followup_message(&self.ctx, message.id, |r| {
                    Self::_choice(prompt, choices, r);
                    r
                })
                .await
                .unwrap();
        } else {
            self.message = Some(
                self.interaction
                    .create_followup_message(&self.ctx, |r| {
                        Self::_choice(prompt, choices, r);
                        r
                    })
                    .await
                    .unwrap(),
            );
        }
        let Some(interaction) = self.message.as_ref().unwrap()
            .await_component_interaction(self.ctx)
            .timeout(Duration::from_secs(60 * 3))
            .collect_limit(1)
            .await
        else {
            self.interaction.edit_followup_message(&self.ctx, self.message.as_ref().unwrap().id, |r| {
                r.content("Didn't receive a response").components(|c| c)
            }).await.unwrap();
            return None;
        };
        interaction
            .create_interaction_response(&self.ctx, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .unwrap();
        interaction.data.values.get(0).cloned()
    }

    pub async fn confirm(&mut self, prompt: &str) -> Confirmation {
        self.initial().await;
        debug!("prompting for confirmation: {}", prompt);
        if let Some(message) = self.message.as_ref() {
            self.interaction
                .edit_followup_message(&self.ctx, message.id, |r| {
                    Self::_confirm(prompt, r);
                    r
                })
                .await
                .unwrap();
        } else {
            self.message = Some(
                self.interaction
                    .create_followup_message(&self.ctx, |r| {
                        Self::_confirm(prompt, r);
                        r
                    })
                    .await
                    .unwrap(),
            );
        }
        let Some(interaction) = self.message.as_ref().unwrap()
            .await_component_interaction(self.ctx)
            .timeout(Duration::from_secs(60 * 3))
            .collect_limit(1)
            .await
        else {
            self.interaction.edit_followup_message(&self.ctx, self.message.as_ref().unwrap().id, |r| {
                r.content("Didn't receive a response").components(|c| c)
            }).await.unwrap();
            return Confirmation::Timeout;
        };
        interaction
            .create_interaction_response(&self.ctx, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .unwrap();
        if interaction.data.custom_id == "yes" {
            Confirmation::Yes
        } else {
            Confirmation::No
        }
    }
}

impl Interaction<'_> {
    fn _choice<T: ToString + Display>(
        prompt: &str,
        choices: &Vec<(String, T)>,
        r: &mut CreateInteractionResponseFollowup,
    ) {
        r.content(prompt).components(|c| {
            c.create_action_row(|r| {
                r.create_select_menu(|m| {
                    m.custom_id("choice").options(|o| {
                        for choice in choices {
                            o.create_option(|o| {
                                o.label(choice.0.to_string()).value(choice.1.to_string())
                            });
                        }
                        o
                    })
                })
            })
        });
    }

    fn _confirm(prompt: &str, r: &mut CreateInteractionResponseFollowup) {
        r.content(prompt).components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| b.style(ButtonStyle::Danger).label("Yes").custom_id("yes"))
                    .create_button(|b| b.style(ButtonStyle::Primary).label("No").custom_id("no"))
            })
        });
    }
}
