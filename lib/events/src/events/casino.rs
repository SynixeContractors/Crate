pub mod db {
    use serde::{Deserialize, Serialize};
    use serenity::all::UserId;
    use synixe_proc::events_requests;

    /// A member's record at one casino game
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct GameStats {
        pub game: String,
        /// Bets placed
        pub bets: i64,
        /// Total bet
        pub wagered: i64,
        /// Total paid out, stakes included
        pub paid_out: i64,
        /// Bets that paid out
        pub wins: i64,
        /// The largest single payout
        pub biggest_win: i64,
    }

    events_requests!(db.casino {
        /// A member's record at each casino game they've played
        struct History {
            member: UserId,
        } => (Result<Vec<GameStats>, String>)
        /// A player buys into a game
        struct BuyIn {
            member: UserId,
            game: String,
            amount: i32,
        } => (Result<(), String>)
        /// A player cashes out of a game
        struct CashOut {
            member: UserId,
            game: String,
            amount: i32,
        } => (Result<(), String>)
    });
}
