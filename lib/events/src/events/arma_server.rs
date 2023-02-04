//! Events for the Arma Server

/// Inform services about the Arma Server.
pub mod publish {
    use synixe_proc::events_publish;

    events_publish!(publish.arma_server {
        /// The server status has changed
        struct Wake {
            /// The server ID
            id: String,
        }
        /// A heartbeat
        struct Heartbeat {
            /// The server ID
            id: String,
        }
    });
}
