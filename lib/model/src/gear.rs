//! Gear, Bank, Shop, Locker

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Price information for an item
pub struct Price {
    personal: i32,
    company: i32,
    personal_current: Option<i32>,
    company_current: Option<i32>,
    until: Option<OffsetDateTime>,
}

impl Price {
    #[must_use]
    /// Create a new price
    pub const fn new(
        personal: i32,
        company: i32,
        personal_current: Option<i32>,
        company_current: Option<i32>,
        until: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            personal,
            company,
            personal_current,
            company_current,
            until,
        }
    }

    #[must_use]
    /// Price of the item
    pub fn personal(&self) -> i32 {
        self.personal_current.unwrap_or(self.personal)
    }

    #[must_use]
    /// Is the item on sale?
    pub const fn personal_base(&self) -> i32 {
        self.personal
    }

    #[must_use]
    /// Price of the item for the company
    pub fn company(&self) -> i32 {
        self.company_current.unwrap_or(self.company)
    }

    #[must_use]
    /// Is the item on sale for the company?
    pub const fn company_base(&self) -> i32 {
        self.company
    }

    #[must_use]
    /// When does the price change end?
    pub const fn until(&self) -> Option<OffsetDateTime> {
        self.until
    }
}

#[cfg(feature = "arma-rs")]
impl arma_rs::IntoArma for Price {
    fn to_arma(&self) -> arma_rs::Value {
        arma_rs::Value::Array(vec![
            self.personal_current.to_arma(),
            self.company_current.to_arma(),
            self.personal.to_arma(),
            self.company.to_arma(),
        ])
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
/// An item from a family of items
pub struct FamilyItem {
    /// The item's family
    pub family: String,
    /// The item's class
    pub class: String,
    /// The item's display name
    pub pretty: String,
}
