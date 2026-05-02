use async_trait::async_trait;
use async_nats::{Client, Message};
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
    ) -> Result<(), anyhow::Error>;
}
