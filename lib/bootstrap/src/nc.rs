use std::sync::Arc;

use nats::asynk::Connection;
use tokio::sync::OnceCell;

type NatsConn = Arc<Connection>;

pub struct NC();

impl NC {
    /// Gets a reference to the NATS connection.
    ///
    /// # Panics
    ///
    /// Panics if the NATS connection can not be initialized.
    pub async fn get() -> NatsConn {
        static NATS: OnceCell<NatsConn> = OnceCell::const_new();
        NATS.get_or_init(|| async {
            Arc::new(
                nats::asynk::connect(
                    std::env::var("NATS_URL").expect("Expected the NATS_URL in the environment"),
                )
                .await
                .expect("Failed to connect to NATS"),
            )
        })
        .await
        .clone()
    }
}
