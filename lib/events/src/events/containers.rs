//! Events for Docker containers

/// Control the docker containers
pub mod docker {
    use synixe_meta::docker::Container;
    use synixe_proc::events_requests;

    events_requests!(docker.containers {
        /// Restart
        struct Restart {
            /// The container
            container: Container,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Start
        struct Start {
            /// The container
            container: Container,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Stop
        struct Stop {
            /// The container
            container: Container,
            /// The reason
            reason: String,
        } => (Result<(), String>)
    });
}
