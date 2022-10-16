use std::{mem::MaybeUninit, sync::Arc};

use sqlx::postgres::PgPoolOptions;

pub type DBPool = std::sync::Arc<sqlx::Pool<sqlx::Postgres>>;
pub struct DB();

impl DB {
    pub async fn get() -> DBPool {
        static mut SINGLETON: MaybeUninit<DBPool> = MaybeUninit::uninit();
        static mut INIT: bool = false;

        unsafe {
            if !INIT {
                SINGLETON.write(Arc::new(
                    PgPoolOptions::new()
                        .min_connections(1)
                        .max_connections(5)
                        .connect(
                            &std::env::var("DATABASE_URL")
                                .expect("Expected the DATABASE_URL in the environment"),
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
