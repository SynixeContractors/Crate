use serde::Deserialize;
use serenity::{
    builder::CreateApplicationCommand,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::*,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("meme")
        .description("I will look for a hot reddit meme")
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let meme = loop {
        let meme: MemeResponse =
            reqwest::get("https://meme-api.com/gimme/memes+dankmemes+armamemes")
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
        if meme.nsfw {
            continue;
        }
        break meme;
    };

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(meme.url))
        })
        .await
    {
        error!("Cannot respond to slash command: {}", why);
    }
}

#[derive(Deserialize)]
struct MemeResponse {
    nsfw: bool,
    url: String,
}
