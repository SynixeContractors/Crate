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

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) -> serenity::Result<()> {
    let mut retry = 2;
    let meme = loop {
        if retry == 0 {
            error!("Cannot get meme");
            break MemeResponse {
                nsfw: false,
                url: "https://media.discordapp.net/attachments/736790994833506414/975213152709013514/unknown.png".to_string(),
            };
        }
        retry -= 1;
        let meme: MemeResponse = {
            let Ok(res) = reqwest::get("https://meme-api.com/gimme/memes+dankmemes+armamemes")
                .await else {
                    error!("Cannot get meme");
                    continue;
                };
            let Ok(json) = res.json().await else {
                error!("Cannot parse meme");
                continue;
            };
            json
        };
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
    Ok(())
}

#[derive(Deserialize)]
struct MemeResponse {
    nsfw: bool,
    url: String,
}
