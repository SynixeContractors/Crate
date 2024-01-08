//! Events for interacting with Discord

/// Write to Discord
pub mod write {
    use serde::{Deserialize, Serialize};
    use serenity::{
        builder::CreateEmbed,
        model::prelude::{ChannelId, ReactionType, RoleId, UserId},
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
                embed = embed.title(title);
            }
            if let Some(description) = val.description {
                embed = embed.description(description);
            }
            if let Some(url) = val.url {
                embed = embed.url(url);
            }
            if let Some(colour) = val.colour {
                embed = embed.color(colour);
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
        /// Ensure a member has a role
        struct EnsureRoles {
            /// The member to check
            member: UserId,
            /// The roles to check
            roles: Vec<RoleId>,
        } => (Result<(), String>)
        /// Write a message to the audit log
        struct Audit {
            /// The message to audit
            message: DiscordMessage,
        } => (Result<(), String>)
        /// Write a message to the game audit log
        struct GameAudit {
            /// The message to audit
            message: DiscordMessage,
        } => (Result<(), String>)
    });
}

/// Get information from Discord
pub mod info {
    use serde::{Deserialize, Serialize};
    use serenity::model::prelude::{Member, RoleId, UserId};
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
        /// Get a member's roles
        struct MemberRoles {
            /// The user to get the roles for
            user: UserId,
        } => (Result<Vec<RoleId>, String>)
        /// Get a member by name
        struct MemberByName {
            /// The name of the user to get
            name: String,
        } => (Result<Option<UserId>, String>)
        /// Get members by role
        struct MembersByRole {
            /// The role to get the members for
            role: RoleId,
        } => (Result<Vec<Member>, String>)
    });
}

/// Read from the database
pub mod db {
    use serenity::model::prelude::UserId;
    use synixe_proc::events_requests;

    events_requests!(db.discord {
        /// Get a discord user's ID from their steam id
        struct FromSteam {
            /// The steam ID to get the discord ID for
            steam: String,
        } => (Result<Option<String>, String>)
        /// Save Steam ID to Database
        struct SaveSteam {
            /// The steam ID to save
            steam: String,
            /// The member to link with
            member: UserId,
        } => (Result<(), String>)
        /// Save owned DLC
        struct SaveDLC {
            /// The steam ID to save the DLC for
            member: UserId,
            /// The DLC to save
            dlc: Vec<u32>,
        } => (Result<(), String>)
        /// Get Active Members
        struct ActiveMembers {} => (Result<Vec<String>, String>)
    });
}

/// Publish event from discord
pub mod publish {
    use serenity::all::GuildMemberUpdateEvent;
    use synixe_proc::events_publish;
    events_publish!(publish.info {
        /// A member was updated
        struct MemberUpdate {
            /// Member that was updated
            member: GuildMemberUpdateEvent,
        }
    });
}

/// Execute event from discord
pub mod executions {
    use synixe_proc::events_requests;
    events_requests!(execute.info {
        /// Update activity roles
        struct UpdateActivityRoles {} => (Result<(), String>)
    });
}
