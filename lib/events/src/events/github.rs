//! Events for GitHub interactions

/// Interact with the database
pub mod db {
    use serenity::{all::ChannelId, model::prelude::UserId};
    use synixe_proc::events_requests;

    events_requests!(db.github {
        /// Find a Discord user by their GitHub username
        struct UserByGitHub {
            /// The member to check
            github: String,
        } => (Result<Option<String>, String>)
        /// Find a GitHub user by their Discord ID
        struct UserByDiscord {
            /// The member to check
            discord: UserId,
        } => (Result<Option<String>, String>)
        /// Link a GitHub user to a Discord user
        struct Link {
            /// The Discord user
            discord: UserId,
            /// The GitHub user
            github: String,
        } => (Result<(), String>)

        /// Save a thread for a pull request
        struct SavePullRequestThread {
            /// The pull request number
            number: i32,
            /// The thread ID
            thread_id: ChannelId,
        } => (Result<(), String>)
        /// Get the thread for a pull request
        struct GetPullRequestThread {
            /// The pull request number
            number: i32,
        } => (Result<Option<String>, String>)
    });
}

/// Inform services about missions.
pub mod publish {
    use synixe_proc::events_publish;

    events_publish!(publish.webhook {
        /// A webhook was received from GitHub
        struct Hook {
            /// The raw JSON data from the webhook
            data: String,
        }
    });
}
