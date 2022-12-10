use std::sync::Arc;

use async_trait::async_trait;
use nats::asynk::{Connection, Message};
use synixe_events::Evokable;

#[async_trait]
/// Handle all events
pub trait Handler: Evokable {
    /// Handle an event
    async fn handle(
        &self,
        msg: Message,
        nats: Arc<Connection>,
    ) -> Result<(), anyhow::Error>;
}
