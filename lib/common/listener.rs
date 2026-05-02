use bootstrap::async_trait::async_trait;
use bootstrap::async_nats::{Client, Message};
use synixe_events::Publishable;
use bootstrap::tokio_stream::StreamExt;

#[async_trait]
/// Lister to all events
pub trait Listener: Publishable {
    /// Listen to an event
    async fn listen(
        &self,
        msg: Message,
        nats: Client,
    ) -> Result<(), bootstrap::anyhow::Error>;
}
