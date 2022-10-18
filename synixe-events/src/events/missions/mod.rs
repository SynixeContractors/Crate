//! Scheduling and running missions.

/// Interact with the database.
pub mod db {
    use chrono::NaiveDateTime;
    use synixe_model::missions::{Mission, ScheduledMission};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.missions {
        /// Schedule a mission
        Schedule {
            /// The mission to schedule.
            mission: String,
            /// The day to schedule the mission.
            date: NaiveDateTime
        } => (Result<(), String>)
        /// Checks if a day is already scheduled.
        IsScheduled {
            /// The day to check.
            date: NaiveDateTime
        } => (Result<Option<bool>, String>)
        /// Gets all upcoming missions.
        UpcomingSchedule {} => (Result<Vec<ScheduledMission>, String>)
        /// Post an upcoming mission
        Post {
            /// The scheduled mission to post.
            mission: Uuid,
        } => (Result<(), String>)
        /// Update missions from the GitHub list
        UpdateMissionList {} => (Result<(), String>)
        /// Fetch the list of missions
        FetchMissionList {} => (Result<Vec<Mission>, String>)
        /// Fetch a single mission
        FetchMission {
            /// The mission to fetch.
            mission: String,
        } => (Result<Option<Mission>, String>)
    });
}
