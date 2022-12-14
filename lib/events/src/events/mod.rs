//! Event Definitions

pub mod arma_server;
pub mod certifications;
pub mod discord;
pub mod gear;
pub mod missions;
pub mod recruiting;

/// Global events
pub mod global {
    use synixe_proc::events_publish;

    events_publish!(publish.global {
        /// Tick event, every 60 seconds
        struct Tick {
            /// The current time
            time: time::OffsetDateTime,
        }
    });
}
