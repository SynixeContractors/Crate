//! Gear, Bank, Shop, Locker

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Price information for an item
pub struct Price {
    base: i32,
    current: Option<i32>,
    until: Option<OffsetDateTime>,
    global: bool,
}

impl Price {
    #[must_use]
    /// Create a new price
    pub const fn new(
        base: i32,
        current: Option<i32>,
        until: Option<OffsetDateTime>,
        global: bool,
    ) -> Self {
        Self {
            base,
            current,
            until,
            global,
        }
    }

    #[must_use]
    /// Price of the item
    pub fn price(&self) -> i32 {
        self.current.unwrap_or(self.base)
    }

    #[must_use]
    /// Is the item on sale?
    pub const fn base(&self) -> bool {
        self.current.is_none()
    }

    #[must_use]
    /// When does the price change end?
    pub const fn until(&self) -> Option<OffsetDateTime> {
        self.until
    }

    #[must_use]
    /// Is the item global?
    pub const fn global(&self) -> bool {
        self.global
    }
}
