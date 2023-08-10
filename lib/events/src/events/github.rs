//! Events for GitHub interactions

/// Interact with the database
pub mod db {
    use serenity::model::prelude::UserId;
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
    });
}

/// Interact with GitHub
pub mod executions {
    use synixe_proc::events_requests;

    events_requests!(executor.github {
        /// Invite a user to the GitHub organization
        struct Invite {
            /// The GitHub user
            github: String,
        } => (Result<(), String>)
    });
}
