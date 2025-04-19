use std::{fmt::Display, time::Duration};

use serenity::{
    all::{ButtonStyle, CommandDataOption, ComponentInteractionDataKind, InteractionType, Message},
    builder::{
        CreateActionRow, CreateButton, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateSelectMenu,
        CreateSelectMenuKind, CreateSelectMenuOption,
    },
    prelude::Context,
};

mod confirm;
pub use confirm::Confirmation;

use crate::get_option;

#[allow(clippy::module_name_repetitions)]
pub trait IntoInteraction {
    fn into_interaction(self) -> serenity::all::Interaction;
}

pub struct Interaction<'a> {
    message: Option<Message>,
    ctx: &'a Context,
    serenity: serenity::all::Interaction,
    initial_response: bool,
    ephemeral: bool,
}

impl<'a> Interaction<'a> {
    /// Only accepts Comand, Component, and Modal interactions
    pub fn new(
        ctx: &'a Context,
        interaction: impl IntoInteraction,
        options: &[CommandDataOption],
    ) -> Self {
        let interaction = interaction.into_interaction();
        if ![
            InteractionType::Command,
            InteractionType::Modal,
            InteractionType::Component,
        ]
        .contains(&interaction.kind())
        {
            panic!("Interaction type must be Command, Modal, or Component");
        }
        Self {
            message: None,
            serenity: interaction,
            ctx,
            initial_response: false,
            ephemeral: !get_option!(options, "public", Boolean).unwrap_or(&false),
        }
    }

    async fn internal_followup(
        &mut self,
        content: CreateInteractionResponseFollowup,
    ) -> serenity::Result<()> {
        if let Some(message) = self.message.as_ref() {
            match &self.serenity {
                serenity::all::Interaction::Command(command) => {
                    command.edit_followup(self.ctx, message.id, content).await?;
                }
                serenity::all::Interaction::Component(component) => {
                    component
                        .edit_followup(self.ctx, message.id, content)
                        .await?;
                }
                serenity::all::Interaction::Modal(modal) => {
                    modal.edit_followup(self.ctx, message.id, content).await?;
                }
                _ => unreachable!(),
            }
        } else {
            self.message = Some(match &self.serenity {
                serenity::all::Interaction::Command(command) => {
                    command.create_followup(self.ctx, content).await?
                }
                serenity::all::Interaction::Component(component) => {
                    component.create_followup(self.ctx, content).await?
                }
                serenity::all::Interaction::Modal(modal) => {
                    modal.create_followup(self.ctx, content).await?
                }
                _ => unreachable!(),
            });
        }
        Ok(())
    }

    async fn initial(&mut self) -> serenity::Result<()> {
        if !self.initial_response {
            let defer = CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::default().ephemeral(self.ephemeral),
            );
            match &self.serenity {
                serenity::all::Interaction::Command(command) => {
                    command.create_response(self.ctx, defer).await?;
                }
                serenity::all::Interaction::Component(component) => {
                    component.create_response(self.ctx, defer).await?;
                }
                serenity::all::Interaction::Modal(modal) => {
                    modal.create_response(self.ctx, defer).await?;
                }
                _ => unreachable!(),
            }
            self.initial_response = true;
        }
        Ok(())
    }

    pub async fn reply(&mut self, content: impl Display + Send) -> serenity::Result<()> {
        self.initial().await?;
        debug!("replying to interaction: {}", content);
        self.internal_followup(
            CreateInteractionResponseFollowup::default()
                .content(content.to_string())
                .components(vec![]),
        )
        .await?;
        Ok(())
    }

    pub async fn choice<T: ToString + Display + Sync>(
        &mut self,
        prompt: &str,
        choices: &[(String, T)],
    ) -> serenity::Result<Option<String>> {
        self.initial().await?;
        debug!("prompting for choice: {}", prompt);
        self.internal_followup(Self::internal_choice(prompt, choices))
            .await?;
        let Some(interaction) = self
            .message
            .as_ref()
            .expect("message should be set after followup")
            .await_component_interaction(self.ctx)
            .timeout(Duration::from_secs(60 * 3))
            .next()
            .await
        else {
            self.internal_followup(
                CreateInteractionResponseFollowup::default()
                    .content("Didn't receive a response")
                    .components(vec![]),
            )
            .await?;
            return Ok(None);
        };
        interaction
            .create_response(&self.ctx, CreateInteractionResponse::Acknowledge)
            .await?;
        let ComponentInteractionDataKind::StringSelect { values } = interaction.data.kind else {
            return Ok(None);
        };
        Ok(values.first().cloned())
    }

    pub async fn confirm(&mut self, prompt: &str) -> serenity::Result<Confirmation> {
        self.initial().await?;
        debug!("prompting for confirmation: {}", prompt);
        self.internal_followup(Self::_confirm(prompt)).await?;
        let Some(interaction) = self
            .message
            .as_ref()
            .expect("message should be set after followup")
            .await_component_interaction(self.ctx)
            .timeout(Duration::from_secs(60 * 3))
            .next()
            .await
        else {
            self.internal_followup(
                CreateInteractionResponseFollowup::default()
                    .content("Didn't receive a response")
                    .components(vec![]),
            )
            .await?;
            return Ok(Confirmation::Timeout);
        };
        interaction
            .create_response(&self.ctx, CreateInteractionResponse::Acknowledge)
            .await?;
        Ok(if interaction.data.custom_id == "yes" {
            Confirmation::Yes
        } else {
            Confirmation::No
        })
    }
}

impl Interaction<'_> {
    fn internal_choice<T: ToString + Display>(
        prompt: &str,
        choices: &[(String, T)],
    ) -> CreateInteractionResponseFollowup {
        CreateInteractionResponseFollowup::default()
            .content(prompt)
            .components(vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
                "choice",
                CreateSelectMenuKind::String {
                    options: choices
                        .iter()
                        .map(|choice| {
                            CreateSelectMenuOption::new(choice.0.to_string(), choice.1.to_string())
                        })
                        .collect::<Vec<_>>(),
                },
            ))])
    }

    fn _confirm(prompt: &str) -> CreateInteractionResponseFollowup {
        CreateInteractionResponseFollowup::default()
            .content(prompt)
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("yes")
                    .style(ButtonStyle::Danger)
                    .label("Yes"),
                CreateButton::new("no")
                    .style(ButtonStyle::Primary)
                    .label("No"),
            ])])
    }
}

impl IntoInteraction for serenity::all::Interaction {
    fn into_interaction(self) -> serenity::all::Interaction {
        self
    }
}

impl IntoInteraction for serenity::all::CommandInteraction {
    fn into_interaction(self) -> serenity::all::Interaction {
        serenity::all::Interaction::Command(self)
    }
}

impl IntoInteraction for serenity::all::ComponentInteraction {
    fn into_interaction(self) -> serenity::all::Interaction {
        serenity::all::Interaction::Component(self)
    }
}

impl IntoInteraction for serenity::all::ModalInteraction {
    fn into_interaction(self) -> serenity::all::Interaction {
        serenity::all::Interaction::Modal(self)
    }
}
