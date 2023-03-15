//! Certifications

#![allow(clippy::use_self)] // serde false positive

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub roles_required: Vec<String>,
    /// Roles granted on certification
    pub roles_granted: Vec<String>,
    /// Valid period in days
    pub valid_for: Option<i32>,
    /// Certification instructors
    pub instructors: Option<Vec<String>>,
    /// Certification created at
    pub created: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A certification trial object
pub struct CertificationTrial {
    /// Certification trial id
    pub id: Uuid,
    /// Certification id
    pub certification: Uuid,
    /// Trainee id
    pub trainee: String,
    /// Instructor id
    pub instructor: String,
    /// Notes
    pub notes: String,
    /// Passed
    pub passed: bool,
    /// Valid for
    pub valid_for: Option<i32>,
    /// Valid until
    pub valid_until: Option<OffsetDateTime>,
    /// Created at
    pub created: OffsetDateTime,
}
