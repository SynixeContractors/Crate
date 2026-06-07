pub mod db {
    use serenity::all::UserId;
    use synixe_proc::events_requests;

    events_requests!(db.casino {
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
