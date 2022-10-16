#![deny(clippy::unwrap_used)]

use nats::asynk::Message;
use opentelemetry::trace::Tracer;
use synixe_events::{discord::write, respond, Evokable};

use crate::Bot;

pub async fn handle(msg: Message, client: Bot) {
    let ((ev, _), pcx) = synixe_events::parse_data!(msg, write::Request);
    let _span = opentelemetry::global::tracer("bot").start_with_context(ev.name(), &pcx);
    println!("write event: {:?}", ev.name());
    match ev {
        write::Request::ChannelMessage { channel, message } => {
            println!("Sending message to channel {}", channel);
            if let Err(e) = channel
                .send_message(&client.http, |m| match message {
                    write::DiscordMessage::Text(text) => m.content(text),
                    write::DiscordMessage::Embed(embed) => m.set_embed(embed.into()),
                })
                .await
            {
                println!("Failed to send message: {}", e);
                if let Err(e) =
                    respond!(msg, &write::Response::ChannelMessage(Err(e.to_string()))).await
                {
                    println!("Failed to respond to NATS: {}", e);
                }
            } else if let Err(e) = respond!(msg, &write::Response::ChannelMessage(Ok(()))).await {
                println!("Failed to respond to NATS: {}", e);
            }
        }
        write::Request::UserMessage { user, message } => {
            let dm = user.create_dm_channel(&client.http).await;
            match dm {
                Ok(dm) => {
                    if let Err(e) = dm
                        .send_message(&client.http, |m| match message {
                            write::DiscordMessage::Text(text) => m.content(text),
                            write::DiscordMessage::Embed(embed) => m.set_embed(embed.into()),
                        })
                        .await
                    {
                        if let Err(e) =
                            respond!(msg, &write::Response::UserMessage(Err(e.to_string()))).await
                        {
                            println!("Failed to respond to NATS: {}", e);
                        }
                    }
                }
                Err(e) => {
                    if let Err(e) =
                        respond!(msg, &write::Response::UserMessage(Err(e.to_string()))).await
                    {
                        println!("Failed to respond to NATS: {}", e);
                    }
                }
            }
        }
    }
}
