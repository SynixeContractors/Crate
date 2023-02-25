#![deny(missing_docs, missing_debug_implementations)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

//! Internal library for model definitions.

#[cfg(feature = "campaigns")]
pub mod campaigns;
#[cfg(feature = "missions")]
pub mod missions;

#[cfg(feature = "certifications")]
pub mod certifications;

#[cfg(feature = "gear")]
pub mod gear;

#[cfg(feature = "garage")]
pub mod garage;

#[cfg(feature = "reset")]
pub mod reset;
