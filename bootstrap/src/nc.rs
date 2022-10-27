use std::{mem::MaybeUninit, sync::Arc};

use nats::asynk::Connection;

type NatsConn = Arc<Connection>;

pub struct NC();

impl NC {
    /// Gets a reference to the NATS connection.
    ///
    /// # Panics
    ///
    /// Panics if the NATS connection can not be initialized.
    pub async fn get() -> NatsConn {
        static mut SINGLETON: MaybeUninit<NatsConn> = MaybeUninit::uninit();
        static mut INIT: bool = false;

        unsafe {
            if !INIT {
                SINGLETON.write(Arc::new(
                    nats::asynk::connect(
                        std::env::var("NATS_URL")
                            .expect("Expected the NATS_URL in the environment"),
                    )
                    .await
                    .unwrap(),
                ));
                INIT = true;
            }
            SINGLETON.assume_init_ref().clone()
        }
    }
}
