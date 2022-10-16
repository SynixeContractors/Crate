use std::collections::HashMap;

use opentelemetry::propagation::{Extractor, Injector};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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

impl<T> Injector for Wrapper<T> {
    fn set(&mut self, key: &str, value: String) {
        self.metadata.insert(key.to_string(), value);
    }
}

impl<T> Extractor for Wrapper<T> {
    fn get(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(std::string::String::as_str)
    }

    fn keys(&self) -> Vec<&str> {
        self.metadata
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }
}
