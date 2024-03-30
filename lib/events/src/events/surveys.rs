//! Events for surveys

/// Interact with the database3
#[allow(clippy::type_complexity)]
pub mod db {
    use serenity::all::UserId;
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.surveys {
        /// Search for a survey
        struct SearchSurvey {
            /// The title to search for
            title: Option<String>,
        } => (Result<Vec<(Uuid, String)>, String>)
        /// Get a survey
        struct GetSurvey {
            /// The survey to get
            survey: Uuid,
        } => (Result<Option<(Uuid, String, String)>, String>)
        /// Get the options from a survey
        struct GetOptions {
            /// The survey to get the options from
            survey: Uuid,
        } => (Result<Vec<String>, String>)
        /// Submit an entry to a survey
        struct Submit {
            /// The survey to submit to
            survey: Uuid,
            /// member id
            member: UserId,
            /// The option to submit
            option: String,
        } => (Result<(), String>)
    });
}
