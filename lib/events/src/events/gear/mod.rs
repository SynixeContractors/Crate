//! Events for persistent gear

/// Interact with the database
pub mod db {
    use std::collections::HashMap;

    use serenity::model::prelude::UserId;
    use synixe_model::gear::Price;
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.gear {
        /// Get a member's loadout
        struct LoadoutGet {
            /// The member's ID
            member: UserId,
        } => (Result<String, String>)
        /// Check if a post has been seen
        struct LoadoutSet {
            /// The member's ID
            member: UserId,
            /// The loadout to set
            loadout: String,
        } => (Result<(), String>)

        /// Get all items stored in a member's locker
        struct LockerGet {
            /// The member's ID
            member: UserId,
        } => (Result<HashMap<String, i32>, String>)
        /// Store items in a member's locker
        struct LockerStore {
            /// The member's ID
            member: UserId,
            /// The items to store
            items: HashMap<String, i32>,
        } => (Result<(), String>)
        /// Take items from a member's locker
        struct LockerTake {
            /// The member's ID
            member: UserId,
            /// The items to take
            items: HashMap<String, i32>,
        } => (Result<(), String>)

        /// Get a member's bank balance
        struct BankBalance {
            /// The member's ID
            member: UserId,
        } => (Result<i64, String>)
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
        /// Purchase an item from the shop
        struct BankPurchasesNew {
            /// The member's ID
            member: UserId,
            /// The item to purchase
            items: Vec<(String, i32, bool)>,
        } => (Result<(), String>)

        /// Get all items in the shop
        struct ShopGetAll {} => (Result<HashMap<String, Price>, String>)
        /// Get the price of an item in the shop
        struct ShopGetPrice {
            /// The item to get the price of
            item: String,
        } => (Result<Price, String>)
    });
}
