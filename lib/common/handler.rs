use async_trait::async_trait;
use async_nats::{Client, Message};
use synixe_events::Evokable;
use bootstrap::tokio_stream::StreamExt;

#[async_trait]
/// Handle all events
pub trait Handler: Evokable {
    /// Handle an event
    async fn handle(
        &self,
        msg: Message,
        nats: Client,
    ) -> Result<(), anyhow::Error>;
}
