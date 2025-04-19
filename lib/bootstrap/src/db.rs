use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tokio::sync::OnceCell;

pub type DBPool = std::sync::Arc<sqlx::Pool<sqlx::Postgres>>;
pub struct DB();

// use OnceCell
impl DB {
    /// Gets a reference to the database pool.
    ///
    /// # Panics
    ///
    /// Panics if the database pool can not be initialized.
    pub async fn get() -> DBPool {
        static DB: OnceCell<DBPool> = OnceCell::const_new();
        DB.get_or_init(|| async {
            Arc::new(
                PgPoolOptions::new()
                    .min_connections(1)
                    .max_connections(5)
                    .connect(
                        &std::env::var("DATABASE_URL")
                            .expect("Expected the DATABASE_URL in the environment"),
                    )
                    .await
                    .expect("should be able to create the database pool"),
            )
        })
        .await
        .clone()
    }
}
