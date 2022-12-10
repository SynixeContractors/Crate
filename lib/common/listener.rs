use std::sync::Arc;

use async_trait::async_trait;
use nats::asynk::{Connection, Message};
use synixe_events::Publishable;

#[async_trait]
/// Lister to all events
pub trait Listener: Publishable {
    /// Listen to an event
    async fn listen(
        &self,
        msg: Message,
        nats: Arc<Connection>,
    ) -> Result<(), anyhow::Error>;
}
