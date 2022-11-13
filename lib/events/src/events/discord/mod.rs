//! Events for interacting with Discord

/// Write to Discord
pub mod write {
    use serde::{Deserialize, Serialize};
    use serenity::{
        builder::CreateEmbed,
        model::prelude::{ChannelId, ReactionType, UserId},
    };
    use synixe_proc::events_requests;

    #[derive(Debug, Serialize, Deserialize)]
    /// A message to be created in Discord
    pub struct DiscordMessage {
        /// Content of the message
        pub content: DiscordContent,
        /// Reactions to add to the message
        pub reactions: Vec<ReactionType>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    /// A message to post to Discord
    pub enum DiscordContent {
        /// A simple message
        Text(String),
        /// A message with an embed
        Embed(DiscordEmbed),
    }

    #[derive(Debug, Serialize, Deserialize)]
    /// A thread to be created in Discord
    pub struct DiscordThread {
        /// Name of the thread
        pub name: String,
        /// Messages to post in the thread
        pub messages: Vec<DiscordMessage>,
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
        struct ChannelMessage {
            /// The channel to send the message to
            channel: ChannelId,
            /// The message to send
            message: DiscordMessage,
            /// Create a thread on the message
            thread: Option<DiscordThread>,
        } => (Result<(), String>)
        /// Direct message a user
        struct UserMessage {
            /// The user to send the message to
            user: UserId,
            /// The message to send
            message: DiscordMessage,
        } => (Result<(), String>)
    });
}

/// Get information from Discord
pub mod info {
    use serde::{Deserialize, Serialize};
    use serenity::model::prelude::{RoleId, UserId};
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
        /// Get a user's name information
        struct Username {
            /// The user to get the name information for
            user: UserId,
        } => (Result<Username, String>)
        /// Get a user's roles
        struct Roles {
            /// The user to get the roles for
            user: UserId,
        } => (Result<Vec<RoleId>, String>)
        /// Get the user by name
        struct UserByName {
            /// The name of the user to get
            name: String,
        } => (Result<Option<UserId>, String>)
    });
}

/// Read from the database
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_proc::events_requests;

    events_requests!(db.discord {
        /// Get a discord user's ID from their name
        struct FromSteam {
            /// The steam ID to get the discord ID for
            steam_id: String,
        } => (Result<String, String>)
        /// Save Steam ID to Database
        struct SaveSteam {
            /// The steam ID to save
            steam_id: String,
            /// The member to link with
            member: UserId,
        } => (Result<(), String>)
    });
}

/// Publish event from discord
pub mod publish {
    use serenity::model::prelude::{Member, Reaction};
    use synixe_proc::events_publish;
    events_publish!(publish.info {
        /// A reaction was added to a message
        struct ReactionAdd {
            /// Reaction added the message
            reaction: Reaction
        }
        /// A reaction was removed from a message
        struct ReactionRemove {
            /// Reaction removed from the message
            reaction: Reaction
        }
        /// A member was updated
        struct MemberUpdate {
            /// Member that was updated
            member: Member,
        }
    });
}
