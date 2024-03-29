//! events for game log

/// Database events
pub mod db {
    use serenity::{all::MessageId, model::prelude::UserId};
    use synixe_proc::events_requests;
    use time::OffsetDateTime;

    events_requests!(db.reputation {
        // Negative reputation events

        /// A friendly unit was shot by a player
        struct FriendlyShot {
            /// The player who shot the friendly unit
            member: UserId,
            /// The friendly unit that was shot
            target: String,
            /// The weapon used to shoot the friendly unit
            weapon: String,
        } => (Result<(), String>)
        /// A civilian was shot by a player
        struct CivilianShot {
            /// The player who shot the civilian
            member: UserId,
            /// The civilian that was shot
            target: String,
            /// The weapon used to shoot the civilian
            weapon: String,
        } => (Result<(), String>)
        /// An unarmed unit was shot by a player
        struct UnarmedShot {
            /// The player who shot the unarmed unit
            member: UserId,
            /// The unarmed unit that was shot
            target: String,
            /// The weapon used to shoot the unarmed unit
            weapon: String,
        } => (Result<(), String>)
        /// A surrendering unit was shot by a player
        struct SurrenderingShot {
            /// The player who shot the surrendering unit
            member: UserId,
            /// The surrendering unit that was shot
            target: String,
            /// The weapon used to shoot the surrendering unit
            weapon: String,
        } => (Result<(), String>)
        /// A captive unit was shot by a player
        struct CaptiveShot {
            /// The player who shot the captive unit
            member: UserId,
            /// The captive unit that was shot
            target: String,
            /// The weapon used to shoot the captive unit
            weapon: String,
        } => (Result<(), String>)
        /// An unconscious unit was shot by a player
        struct UnconsciousShot {
            /// The player who shot the unconscious unit
            member: UserId,
            /// The unconscious unit that was shot
            target: String,
            /// The weapon used to shoot the unconscious unit
            weapon: String,
        } => (Result<(), String>)
        /// A building was damaged by a player
        struct BuildingDamaged {
            /// The player who damaged the building
            member: UserId,
            /// The building that was damaged
            target: String,
            /// The weapon used to damage the building
            weapon: String,
        } => (Result<(), String>)

        // Positive reputation events

        /// A friendly unit was healed by a player
        struct FriendlyHealed {
            /// The player who healed the friendly unit
            member: UserId,
            /// The friendly unit that was healed
            target: String,
        } => (Result<(), String>)
        /// An unfriendly unit was healed by a player
        struct UnfriendlyHealed {
            /// The player who healed the unfriendly unit
            member: UserId,
            /// The unfriendly unit that was healed
            target: String,
        } => (Result<(), String>)
        /// A civilian was healed by a player
        struct CivilianHealed {
            /// The player who healed the civilian
            member: UserId,
            /// The civilian that was healed
            target: String,
        } => (Result<(), String>)

        // Other events

        /// A mission was completed
        struct MissionCompleted {
            /// The player who completed the mission
            member: UserId,
            /// The mission that was completed
            mission: String,
            /// Reputation change
            reputation: i32,
        } => (Result<(), String>)
        /// Get the current reputation of the group
        /// This function should never return None
        struct CurrentReputation {
            /// The time to get the reputation for
            at: OffsetDateTime,
        } => (Result<Option<Option<f64>>, String>)
        /// Update reputation
        struct UpdateReputation {
            /// Staff member updating the reputation
            staff: UserId,
            /// The player who caused the reputation
            member: UserId,
            /// The amount of reputation to remove
            reputation: i32,
            /// The reason for the reputation
            reason: String,
        } => (Result<(), String>)

        // Admin events

        /// Delete a reputation event
        struct DeleteByMessage {
            /// The message to delete
            message: MessageId,
        } => (Result<(), String>)
    });
}
