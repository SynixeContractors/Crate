//! Scheduling and running missions.

#![allow(clippy::use_self)]

/// Interact with the database.
pub mod db {
    use serenity::model::prelude::MessageId;
    use synixe_model::missions::{Mission, MissionRsvp, Rsvp, ScheduledMission};
    use synixe_proc::events_requests;
    use time::OffsetDateTime;
    use uuid::Uuid;

    events_requests!(db.missions {
        /// Schedule a mission
        struct Schedule {
            /// The mission to schedule.
            mission: String,
            /// The day to schedule the mission.
            date: OffsetDateTime,
        } => (Result<(), String>)
        /// Checks if a day is already scheduled.
        struct IsScheduled {
            /// The day to check.
            date: OffsetDateTime,
        } => (Result<Option<Option<bool>>, String>)
        /// Remove a scheduled mission.
        struct Unschedule {
            /// The mission to remove.
            scheduled_mission: Uuid,
        } => (Result<(), String>)
        /// Set the message for a scheduled mission.
        struct SetScheduledMesssage {
            /// The scheduled mission to update.
            scheduled_mission: Uuid,
            /// The message in #schedule
            schedule_message_id: String,
        } => (Result<(), String>)
        /// Gets all upcoming missions.
        struct UpcomingSchedule {} => (Result<Vec<ScheduledMission>, String>)
        /// Update missions from the GitHub list
        struct UpdateMissionList {} => (Result<(), String>)
        /// Fetch the list of missions
        struct FetchMissionList {
            /// Filter the list of missions by name
            search: Option<String>,
        } => (Result<Vec<Mission>, String>)
        /// Fetch a single mission
        struct FetchMission {
            /// The mission to fetch.
            mission: String,
        } => (Result<Option<Mission>, String>)
        /// Fetch a single scheduled mission by it's message id
        struct FetchScheduledMission {
            /// The message id to fetch.
            message: MessageId,
        } => (Result<Option<ScheduledMission>, String>)
        /// Fetch the RSVPs for a mission
        struct FetchMissionRsvps {
            /// The mission to fetch.
            mission: String,
        } => (Result<Vec<MissionRsvp>, String>)
        /// Add an RSVP to a mission
        struct AddMissionRsvp {
            /// The mission to RSVP to.
            mission: String,
            /// The user to RSVP.
            member: String,
            /// Their RSVP.
            rsvp: Rsvp,
            /// Extra details
            details: Option<String>,
        } => (Result<(), String>)
    });
}

/// Interact with executor
pub mod executions {
    use synixe_proc::events_requests;

    events_requests!(executor.missions {
        /// Post about upcoming missions
        struct PostUpcomingMissions {} => (Result<(), String>)
    });
}

/// Inform services about missions.
pub mod publish {
    use synixe_model::missions::{Mission, MissionType, ScheduledMission};
    use synixe_proc::events_publish;

    events_publish!(publish.missions {
        /// Publish a scheduled mission
        struct StartingSoon {
            /// The  mission starting soon
            mission: Mission,
            /// The schedule mission
            scheduled: ScheduledMission,
            /// Minutes until the mission starts
            minutes: i64,
        }
        /// It's time to load a mission on a server
        struct ChangeMission {
            /// The mission to load
            id: String,
            /// Type of mission
            mission_type: MissionType,
        }
        /// Warn that the mission is about to change
        struct WarnChangeMission {
            /// The mission to load
            id: String,
            /// Type of mission
            mission_type: MissionType,
        }
    });
}
