//! Scheduling and running missions.

/// Interact with the database.
pub mod db {
    use chrono::NaiveDateTime;
    use synixe_model::missions::{Mission, ScheduledMission};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.missions {
        /// Schedule a mission
        struct Schedule {
            /// The mission to schedule.
            mission: String,
            /// The day to schedule the mission.
            date: NaiveDateTime,
        } => (Result<(), String>)
        /// Checks if a day is already scheduled.
        struct IsScheduled {
            /// The day to check.
            date: NaiveDateTime,
        } => (Result<Option<bool>, String>)
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
        struct FetchMissionList {} => (Result<Vec<Mission>, String>)
        /// Fetch a single mission
        struct FetchMission {
            /// The mission to fetch.
            mission: String,
        } => (Result<Option<Mission>, String>)
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
    use chrono::NaiveDateTime;
    use synixe_model::missions::{Mission, MissionType};
    use synixe_proc::events_publish;

    events_publish!(publish.missions {
        /// Publish a scheduled mission
        struct StartingSoon {
            /// The  mission starting soon
            mission: Mission,
            /// Start time
            start_time: NaiveDateTime,
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
