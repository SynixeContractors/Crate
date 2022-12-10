use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Wrapper around an event
pub struct Wrapper<T> {
    #[serde(rename = "i")]
    inner: T,
    #[serde(rename = "m")]
    metadata: HashMap<String, String>,
}

impl<T> Wrapper<T> {
    /// Create a new wrapper
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            metadata: HashMap::new(),
        }
    }

    /// Get the inner value
    pub const fn inner(&self) -> &T {
        &self.inner
    }

    /// Get the metadata
    pub const fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Into the parts
    #[allow(clippy::missing_const_for_fn)] // False positive
    pub fn into_parts(self) -> (T, HashMap<String, String>) {
        (self.inner, self.metadata)
    }
}
