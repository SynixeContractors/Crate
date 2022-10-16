//! Events for interacting with Discord

/// Write to Discord
pub mod write {
    use serde::{Deserialize, Serialize};
    use serenity::{
        builder::CreateEmbed,
        model::prelude::{ChannelId, Message, UserId},
    };
    use synixe_proc::events_requests;

    #[derive(Debug, Serialize, Deserialize)]
    /// A message to post to Discord
    pub enum DiscordMessage {
        /// A simple message
        Text(String),
        /// A message with an embed
        Embed(DiscordEmbed),
    }

    #[derive(Debug, Serialize, Deserialize)]
    /// Limited embed for events
    pub struct DiscordEmbed {
        /// Title of the embed
        pub title: Option<String>,
        /// Description of the embed
        pub description: Option<String>,
        /// URL of the embed
        pub url: Option<String>,
        /// Colour of the embed
        pub colour: Option<i32>,
    }

    impl From<DiscordEmbed> for CreateEmbed {
        fn from(val: DiscordEmbed) -> Self {
            let mut embed = Self::default();
            if let Some(title) = val.title {
                embed.title(title);
            }
            if let Some(description) = val.description {
                embed.description(description);
            }
            if let Some(url) = val.url {
                embed.url(url);
            }
            if let Some(colour) = val.colour {
                embed.color(colour);
            }
            embed
        }
    }

    events_requests!(discord.write {
        /// Send a message to a channel
        ChannelMessage {
            /// The channel to send the message to
            channel: ChannelId,
            /// The message to send
            message: DiscordMessage,
        } => (Result<Message, String>)
        /// Direct message a user
        UserMessage {
            /// The user to send the message to
            user: UserId,
            /// The message to send
            message: DiscordMessage,
        } => (Result<Message, String>)
    });
}

/// Get information from Discord
pub mod info {
    use serde::{Deserialize, Serialize};
    use serenity::model::prelude::UserId;
    use synixe_proc::events_requests;

    #[derive(Debug, Serialize, Deserialize)]
    /// A user's name information
    pub struct Username {
        /// The name to display
        pub response: String,
        /// The user's nickname
        pub nickname: Option<String>,
        /// The user's username
        pub user_name: String,
    }

    events_requests!(discord.info {
        /// A message was sent
        Username {
            /// The message that was sent
            user: UserId,
        } => (Result<Username, String>)
    });
}
