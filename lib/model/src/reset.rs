//! Reset
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// Unclaimed cert kit
pub struct UnclaimedKit {
    /// First Kit ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Specialist
    pub specialist: bool,
}
