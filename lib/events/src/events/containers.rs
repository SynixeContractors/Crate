//! Events for Docker containers

/// Control the docker containers
pub mod docker {
    use synixe_meta::docker::ArmaServer;
    use synixe_proc::events_requests;

    events_requests!(docker.containers {
        /// Restart
        struct Restart {
            /// The server
            server: ArmaServer,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Start
        struct Start {
            /// The server
            server: ArmaServer,
            /// The reason
            reason: String,
        } => (Result<(), String>)
        /// Stop
        struct Stop {
            /// The server
            server: ArmaServer,
            /// The reason
            reason: String,
        } => (Result<(), String>)
    });
}

/// Events for the Swfity modpack
pub mod modpack {
    use synixe_proc::events_requests;

    events_requests!(docker.modpack {
        /// Updated the modpack
        struct Updated {} => (Result<(), String>)
    });
}

/// Events for Missions
pub mod missions {
    use synixe_proc::events_requests;

    events_requests!(docker.missions {
        /// Updated the mission list
        struct UpdateMissionList {} => (Result<(), String>)
    });
}
