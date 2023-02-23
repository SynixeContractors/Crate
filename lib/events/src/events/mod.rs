//! Event Definitions

pub mod campaigns;
pub mod certifications;
pub mod containers;
pub mod discord;
pub mod garage;
pub mod gear;
pub mod missions;
pub mod recruiting;
pub mod servers;

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
