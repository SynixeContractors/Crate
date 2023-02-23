//! Events for servers

/// Interact with the database
pub mod db {
    use synixe_proc::events_requests;

    events_requests!(db.servers {
        /// Log a server event
        struct Log {
            /// The server id
            server: String,
            /// The user id
            steam: String,
            /// The action
            action: String,
            /// The data
            data: serde_json::Value,
        } => (Result<(), String>)
    });
}
