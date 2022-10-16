use std::sync::Arc;

use async_trait::async_trait;
use bootstrap::DBPool;
use nats::asynk::{Connection, Message};
use opentelemetry::Context;
use synixe_events::Evokable;

mod recruiting;

#[async_trait]
pub trait Handler: Evokable {
    async fn handle(
        &self,
        msg: Message,
        nats: Arc<Connection>,
        db: DBPool,
        cx: Context,
    ) -> Result<(), anyhow::Error>;
}
