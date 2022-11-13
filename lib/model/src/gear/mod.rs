//! Gear, Bank, Shop, Locker

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// A deposit object
pub struct Deposit {
    /// The member's ID
    pub member: String,
    /// The amount deposited
    pub amount: i32,
    /// The reason for the deposit
    pub reason: String,
    /// The deposit id
    pub id: Uuid,
    /// The time of the deposit
    pub created: OffsetDateTime,
}

impl Deposit {
    #[must_use]
    /// The member's ID
    pub fn member(&self) -> &str {
        &self.member
    }

    #[must_use]
    /// The amount deposited
    pub const fn amount(&self) -> i32 {
        self.amount
    }

    #[must_use]
    /// The reason for the deposit
    pub fn reason(&self) -> &str {
        &self.reason
    }

    #[must_use]
    /// The deposit ID
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    /// The time the deposit was made
    pub const fn created(&self) -> OffsetDateTime {
        self.created
    }
}
