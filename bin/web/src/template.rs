use std::sync::{Arc, OnceLock};

use tera::Tera;

pub struct Template;

impl Template {
    /// Gets a reference to the Template singleton
    ///
    /// # Panics
    ///
    /// Panics if the bot does not exists
    pub fn get() -> Arc<Tera> {
        static SINGLETON: OnceLock<Arc<Tera>> = OnceLock::new();
        SINGLETON
            .get_or_init(|| {
                Arc::new(Tera::new("templates/**/*.html").expect("Failed to load templates"))
            })
            .clone()
    }
}
