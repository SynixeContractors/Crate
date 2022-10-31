#![deny(missing_docs, missing_debug_implementations)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

//! Internal library for model definitions.

use serde::{Deserialize, Serialize};

#[cfg(feature = "roles")]
use serenity::model::prelude::RoleId;

#[cfg(feature = "missions")]
pub mod missions;

#[cfg(feature = "certifications")]
pub mod certifications;

#[cfg(feature = "roles")]
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// Discord roles
pub struct Roles(Vec<RoleId>);
