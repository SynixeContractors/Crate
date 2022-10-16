//! Events for recruiting

/// Interact with the database
pub mod db {
    use synixe_proc::events_requests;

    events_requests!(db.recruiting {
        /// Mark a post as seen
        Seen {
            /// The post url
            url: String
        } => (Result<(), String>)
        /// Check if a post has been seen
        HasSeen {
            /// The post url
            url: String
        } => (Result<Option<bool>, String>)
    });
}
