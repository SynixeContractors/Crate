//! Certifications

#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Roles;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A certification object
pub struct Certification {
    /// Certification id
    pub id: Uuid,
    /// Certification name
    pub name: String,
    /// Certification link
    pub link: String,
    /// Roles required to certify
    pub roles_required: sqlx::types::Json<Roles>,
    /// Roles granted on certification
    pub roles_granted: sqlx::types::Json<Roles>,
    /// Valid period in days
    pub valid_for: i32,
    /// Certification created at
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A certification trial object
pub struct CertificationTrial {
    /// Certification trial id
    pub id: Uuid,
    /// Certification id
    pub certification_id: Uuid,
    /// Trainee id
    pub trainee_id: String,
    /// Instructor id
    pub instructor_id: String,
    /// Notes
    pub notes: String,
    /// Valid until
    pub valid_until: Option<chrono::NaiveDateTime>,
    /// Created at
    pub created_at: chrono::NaiveDateTime,
}
