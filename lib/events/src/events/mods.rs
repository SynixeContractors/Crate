//! events for game log

/// Database events
pub mod db {
    use synixe_model::mods::Workshop;
    use synixe_proc::events_requests;

    events_requests!(db.mods {
        /// check if a mod has been updated
        struct  GetAllMods {
        } => (Result<Vec<Workshop>, String>)
    });
}

/// Execute events.
pub mod executions {
    use synixe_proc::events_requests;

    events_requests!(executor.mods {
        /// check if a mod has been updated
        struct CheckSteamModUpdates {} => (Result<(), String>)
    });
}
