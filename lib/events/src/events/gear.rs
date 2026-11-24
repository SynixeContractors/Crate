//! Events for persistent gear

/// Interact with the database
pub mod db {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serenity::model::prelude::UserId;
    use synixe_model::gear::{Deposit, FamilyItem, Price};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Transaction {
        pub amount: i32,
        pub created: time::OffsetDateTime,
        pub kind: TransactionKind,
        pub reason: String,
        /// Items bought in this transaction, for purchases
        pub items: Vec<PurchasedItem>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PurchasedItem {
        /// The item's pretty name, or class if it has none
        pub name: String,
        pub quantity: i32,
    }

    /// Where a bank request is in its life
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum BankRequestStatus {
        /// Waiting for the payer to pay or decline
        Open,
        /// Being paid right now, so it can't be answered twice
        Paying,
        /// Paid, the money has moved
        Paid,
        /// Declined by the payer
        Declined,
    }

    impl BankRequestStatus {
        #[must_use]
        pub const fn as_str(self) -> &'static str {
            match self {
                Self::Open => "open",
                Self::Paying => "paying",
                Self::Paid => "paid",
                Self::Declined => "declined",
            }
        }

        /// Parses a status as stored in the database
        #[must_use]
        pub fn parse(status: &str) -> Option<Self> {
            match status {
                "open" => Some(Self::Open),
                "paying" => Some(Self::Paying),
                "paid" => Some(Self::Paid),
                "declined" => Some(Self::Declined),
                _ => None,
            }
        }
    }

    /// A member asking another to pay them
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct BankRequest {
        pub id: Uuid,
        pub requester: UserId,
        pub payer: UserId,
        pub amount: i32,
        pub reason: String,
        pub status: BankRequestStatus,
        pub created: time::OffsetDateTime,
        pub resolved: Option<time::OffsetDateTime>,
    }

    /// A transfer of money between two members
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Transfer {
        pub source: UserId,
        pub target: UserId,
        pub amount: i32,
        pub reason: String,
        pub created: time::OffsetDateTime,
    }

    /// How the company has been doing, for Brodsky's mood
    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct CompanyActivity {
        /// The company account's balance
        pub balance: i64,
        /// Money into the company account in the period
        pub income: i64,
        /// Money out of the company account in the period, as a positive amount
        pub spending: i64,
        /// Reputation gained in the period, negative if it was lost
        pub reputation: i64,
        /// Missions in the period that succeeded, partly succeeded and failed
        pub successes: i64,
        pub partials: i64,
        pub failures: i64,
    }

    /// An item in a member's locker
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct LockerItem {
        pub class: String,
        /// The item's pretty name, if it has one
        pub pretty: Option<String>,
        pub quantity: i32,
        /// The base price of one, as the locker's value is counted. `None` if the item has no price.
        pub value: Option<i32>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum TransactionKind {
        Deposit,
        TransferIn,
        TransferOut,
        Purchase,
    }

    impl TransactionKind {
        #[must_use]
        pub const fn label(self) -> &'static str {
            match self {
                Self::Deposit => "deposit",
                Self::TransferIn => "transfer in",
                Self::TransferOut => "transfer out",
                Self::Purchase => "purchase",
            }
        }
    }

    events_requests!(db.gear {
        /// Get a member's loadout
        struct LoadoutGet {
            /// The member's ID
            member: UserId,
            /// The campaign ID, if any
            campaign: Option<Uuid>,
        } => (Result<Option<String>, String>)
        /// Check if a post has been seen
        struct LoadoutStore {
            /// The member's ID
            member: UserId,
            /// The loadout to set
            loadout: String,
            /// The campaign ID, if any
            campaign: Option<Uuid>,
        } => (Result<(), String>)
        /// Get all items stored in a member's locker
        struct LoadoutBalance {
            /// The member's ID
            member: UserId,
        } => (Result<i32, String>)
        /// Get all items stored in a member's locker
        struct LockerGet {
            /// The member's ID
            member: UserId,
        } => (Result<HashMap<String, i32>, String>)
        /// Get balance of items in a member's locker
        struct LockerBalance {
            /// The member's ID
            member: UserId,
        } => (Result<i32, String>)
        /// Store items in a member's locker
        struct LockerStore {
            /// The member's ID
            member: UserId,
            /// The items to store
            items: HashMap<String, i32>,
            /// Reason
            reason: String,
        } => (Result<(), String>)
        /// Take items from a member's locker
        struct LockerTake {
            /// The member's ID
            member: UserId,
            /// The items to take
            items: HashMap<String, i32>,
            /// Reason
            reason: String,
        } => (Result<(), String>)

        /// Get a member's bank balance
        struct BankBalance {
            /// The member's ID
            member: UserId,
        } => (Result<Option<i32>, String>)
        /// Deposit money into a member's bank
        struct BankDepositNew {
            /// The member's ID
            member: UserId,
            /// The amount to deposit
            amount: i32,
            /// The reason for the deposit
            reason: String,
            /// Deposit id
            id: Option<Uuid>,
        } => (Result<(), String>)
        /// Search for a deposit
        struct BankDepositSearch {
            /// The member's ID
            member: UserId,
            /// The deposit id
            id: Option<Uuid>,
            /// The reason for the deposit
            reason: Option<String>,
        } => (Result<Vec<Deposit>, String>)
        /// Transfer money from a member's bank to another member's bank
        struct BankTransferNew {
            /// Source member's ID
            source: UserId,
            /// Target member's ID
            target: UserId,
            /// The amount to transfer
            amount: i32,
            /// The reason for the transfer
            reason: String,
        } => (Result<(), String>)
        /// Get money spent per category for a member
        struct BankSpent {
            /// The member's ID
            member: UserId,
        } => (Result<HashMap<String, i32>, String>)
        /// Get history of spending for a member
        struct BankHistory {
            /// The member's ID
            member: UserId,
        } => (Result<Vec<Transaction>, String>)
        /// What the company has been up to over the last `days`
        struct CompanyActivity {
            days: i32,
        } => (Result<CompanyActivity, String>)
        /// The average price members paid for an item each week over the last year, oldest first
        struct ShopPriceHistory {
            item: String,
        } => (Result<Vec<(time::OffsetDateTime, i32)>, String>)
        /// Get the items in a member's locker, ordered by name
        struct LockerContents {
            /// The member's ID
            member: UserId,
        } => (Result<Vec<LockerItem>, String>)
        /// Get a member's transfers, sent and received, newest first, a page at a time
        struct BankTransfers {
            /// The member's ID
            member: UserId,
            /// How many to return
            limit: i64,
            /// How many newer transfers to skip
            offset: i64,
        } => (Result<Vec<Transfer>, String>)
        /// Record a request for `payer` to pay `requester`. Responds with the request's ID.
        struct CreateBankRequest {
            requester: UserId,
            payer: UserId,
            amount: i32,
            reason: String,
        } => (Result<Uuid, String>)
        /// Move a request from one status to another, only if it's in `from` and `payer` is the one asked.
        /// Responds with the request if it moved, `None` if it wasn't in that status.
        struct TransitionBankRequest {
            id: Uuid,
            payer: UserId,
            from: BankRequestStatus,
            to: BankRequestStatus,
        } => (Result<Option<BankRequest>, String>)
        /// A member's requests, as requester or payer, newest first, a page at a time
        struct BankRequests {
            member: UserId,
            limit: i64,
            offset: i64,
        } => (Result<Vec<BankRequest>, String>)

        /// Get all items in the shop
        #[allow(clippy::type_complexity)]
        struct ShopGetAll {
            page: u32,
        } => (Result<(HashMap<String, (Option<String>, Option<Vec<String>>, Price, bool)>, bool), String>)
        /// Get the price of an item in the shop
        struct ShopGetPrice {
            /// The item to get the price of
            item: String,
        } => (Result<Price, String>)
        /// Search shop items by class or pretty name, exact matches first
        struct ShopSearch {
            /// The text to search for
            search: String,
        } => (Result<Vec<(String, Option<String>)>, String>)

        /// Helper for Arma to:
        /// - Set the loadout to blank
        /// - Store items in the locker
        /// - Get the player's bank balance
        struct ShopEnter {
            /// The member's ID
            member: UserId,
            /// The items to store
            items: HashMap<String, i32>,
            /// The magazines to purchase (bullets needed, bullets per magazine)
            magazines: HashMap<String, (i32, i32)>,
        } => (Result<(HashMap<String, i32>, i32), String>)
        /// Helper for Arma to:
        /// - Set the player's loadout
        /// - Take items from the locker
        struct ShopLeave {
            /// The member's ID
            member: UserId,
            /// The items to take
            items: HashMap<String, i32>,
            /// The loadout to set
            loadout: String,
        } => (Result<(), String>)
        /// Helper for Arma to purchase items from the shop
        struct ShopPurchase {
            /// The member's ID
            member: UserId,
            /// The items to purchase
            items: HashMap<String, i32>,
        } => (Result<(HashMap<String, i32>, i32), String>)

        /// Set the pretty name of an item
        struct SetPrettyName {
            /// The item to set the pretty name of
            item: String,
            /// The pretty name to set
            pretty: String,
        } => (Result<(), String>)
        /// Get the pretty name of an item
        struct GetPrettyName {
            /// The item to get the pretty name of
            item: String,
        } => (Result<Option<Option<String>>, String>)

        /// Find items that share a relation with another item
        struct FamilySearch {
            /// The weapon's class
            item: String,
            /// Relation
            relation: String,
        } => (Result<Vec<FamilyItem>, String>)
        /// Find items that a user has in their locker that are in a family with the given relation
        struct FamilyCompatibleItems {
            /// Member
            member: UserId,
            /// Relation
            relation: String,
        } => (Result<Vec<FamilyItem>, String>)
        /// Repaints an item, taking it from the locker and purchasing the new item
        struct FamilyReplace {
            /// Member
            member: UserId,
            /// Original item
            original: String,
            /// New item
            new: String,
            /// Reason
            reason: String,
            /// Cost
            cost: i32,
        } => (Result<(), String>)
    });
}

/// Inform services about gear.
pub mod publish {
    use synixe_proc::events_publish;

    events_publish!(publish.gear {
        /// A member's balance has changed
        struct BalanceChanged {
            /// The member's ID
            member: serenity::model::prelude::UserId,
            /// The new balance
            new_balance: i32,
            /// The reason for the change
            reason: String,
        }
    });
}
