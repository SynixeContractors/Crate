//! Event Definitions

pub mod campaigns;
pub mod certifications;
pub mod containers;
pub mod discord;
pub mod garage;
pub mod gear;
pub mod github;
pub mod missions;
pub mod recruiting;
pub mod reputation;
// pub mod reset;
pub mod servers;
pub mod surveys;
pub mod voting;

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
