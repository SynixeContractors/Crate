#![deny(missing_docs, missing_debug_implementations)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

//! Internal library for model definitions.

#[cfg(feature = "missions")]
pub mod missions;

#[cfg(feature = "certifications")]
pub mod certifications;

#[cfg(feature = "gear")]
pub mod gear;
