//! Events for persistent gear

/// Interact with the database
pub mod db {
    use std::collections::HashMap;

    use serenity::model::prelude::UserId;
    use synixe_model::gear::{Deposit, FamilyItem, Price};
    use synixe_proc::events_requests;
    use uuid::Uuid;

    events_requests!(db.gear {
        /// Get a member's loadout
        struct LoadoutGet {
            /// The member's ID
            member: UserId,
        } => (Result<Option<String>, String>)
        /// Check if a post has been seen
        struct LoadoutStore {
            /// The member's ID
            member: UserId,
            /// The loadout to set
            loadout: String,
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
        /// Purchase an item from the shop
        struct BankPurchasesNew {
            /// The member's ID
            member: UserId,
            /// The item to purchase
            items: Vec<(String, i32, bool)>,
        } => (Result<(), String>)

        /// Get all items in the shop
        #[allow(clippy::type_complexity)]
        struct ShopGetAll {} => (Result<HashMap<String, (Option<String>, Option<Vec<String>>, Price)>, String>)
        /// Get the price of an item in the shop
        struct ShopGetPrice {
            /// The item to get the price of
            item: String,
        } => (Result<Price, String>)

        /// Helper for Arma to:
        /// - Set the loadout to blank
        /// - Store items in the locker
        /// - Get the player's bank balance
        struct ShopEnter {
            /// The member's ID
            member: UserId,
            /// The items to store
            items: HashMap<String, i32>,
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
