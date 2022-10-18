use serenity::{
    model::prelude::{
        application_command::ApplicationCommandInteraction, InteractionResponseType, RoleId,
    },
    prelude::Context,
};

pub mod meme;
pub mod missions;

pub async fn requires_role(
    needle: RoleId,
    haystack: &[RoleId],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> bool {
    let found = haystack.iter().any(|role| *role == needle);
    if !found {
        command
            .create_interaction_response(&ctx, |r| {
                r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {
                        m.content("You do not have permission to use this command")
                            .ephemeral(true)
                    })
            })
            .await
            .unwrap();
    }
    found
}
