//! Events for Docker containers

/// Control the docker containers
pub mod docker {
    use synixe_proc::events_requests;

    events_requests!(docker.containers {
        /// Restart
        struct Restart {
            /// The server ID
            id: String,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Start
        struct Start {
            /// The server ID
            id: String,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Stop
        struct Stop {
            /// The server ID
            id: String,
            /// The reason
            reason: String,
        } => (Result<(), String>)
    });
}
