use async_nats::Client;
use tokio::sync::OnceCell;

pub use async_nats;

pub struct NC();

impl NC {
    /// Gets a reference to the NATS connection.
    ///
    /// # Panics
    ///
    /// Panics if the NATS connection can not be initialized.
    pub async fn get() -> Client {
        static NATS: OnceCell<Client> = OnceCell::const_new();
        NATS.get_or_init(|| async {
            async_nats::connect(
                std::env::var("NATS_URL").expect("Expected the NATS_URL in the environment"),
            )
            .await
            .expect("Failed to connect to NATS")
        })
        .await
        .clone()
    }
}
