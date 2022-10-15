//! Events for interacting with Discord
pub mod write {
    use serenity::model::prelude::Message;
    use synixe_proc::events_requests;

    events_requests!(discord.write {
        /// Send a message to a channel
        ChannelMessage {
            /// The channel to send the message to
            channel: u64,
            /// The message to send
            message: String,
        } => (Result<Message, String>)
        /// Direct message a user
        UserMessage {
            /// The user to send the message to
            user: u64,
            /// The message to send
            message: String,
        } => (Result<Message, String>)
    });
}
