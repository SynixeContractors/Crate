//! Events for recruiting

/// Interact with the database
pub mod db {
    use synixe_proc::events_requests;

    events_requests!(db.recruiting {
        /// Mark a post as seen
        struct Seen {
            /// The post url
            url: String
        } => (Result<(), String>)
        /// Check if a post has been seen
        struct HasSeen {
            /// The post url
            url: String
        } => (Result<Option<bool>, String>)
        /// Mark a post as replied to
        struct Replied {
            /// The post url
            url: String
        } => (Result<(), String>)
        /// Check if a post has been replied to
        struct HasReplied {
            /// The post url
            url: String
        } => (Result<Option<bool>, String>)
    });
}

/// Interact with the executor
pub mod executions {
    use synixe_proc::events_requests;

    events_requests!(executor.recruiting {
        /// Check for new post on Steam
        struct CheckSteam {} => (Result<(), String>)
        /// Check for new post on Reddit
        struct CheckReddit {} => (Result<(), String>)
        /// Post on Reddit
        struct PostReddit {} => (Result<(), String>)
        /// Reply on Reddit
        struct ReplyReddit {
            /// Link to post
            url: String
        } => (Result<(), String>)
    });
}
