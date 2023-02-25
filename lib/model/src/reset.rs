//! Reset

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// Unclaimed cert kit
pub struct UnclaimedKit {
    /// Cert
    pub id: Uuid,
    /// Name
    pub name: String,
}
