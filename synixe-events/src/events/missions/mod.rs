//! Scheduling and running missions.

/// Interact with the database.
pub mod db {
    use chrono::NaiveDateTime;
    use synixe_model::missions::Mission;
    use synixe_proc::events_requests;

    events_requests!(db.missions {
        /// Schedule a mission
        Schedule {
            /// The mission to schedule.
            mission: String,
            /// The day to schedule the mission.
            date: NaiveDateTime
        } => (Result<(), String>)
        /// Update missions from the GitHub list
        UpdateMissionList {} => (Result<(), String>)
        /// Fetch the list of missions
        FetchMissionList {} => (Result<Vec<Mission>, String>)
    });
}
