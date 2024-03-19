//! Events for voting

/// Interact with the database3
#[allow(clippy::type_complexity)]
pub mod db {
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.voting {
        /// Check if a ticket has been used
        struct CheckTicket {
            /// The ticket to check
            ticket: String,
        } => (Result<bool, String>)
        /// Get a poll
        struct GetPoll {
            /// The poll to get
            poll: Uuid,
        } => (Result<Option<(Uuid, String, String, Option<String>)>, String>)
        /// Get the options from a poll
        struct GetOptions {
            /// The poll to get the options from
            poll: Uuid,
        } => (Result<Vec<(Uuid, String)>, String>)
        /// Cast a vote
        struct Vote {
            /// The poll to vote on
            poll: Uuid,
            /// The ticket to vote with
            ticket: String,
            /// The encrypted option
            option: String,
        } => (Result<(), String>)
    });
}
