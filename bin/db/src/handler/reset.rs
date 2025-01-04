use super::Handler;
use async_trait::async_trait;
use serde_json::{Number, Value};
use std::collections::HashMap;
use synixe_events::{
    reset::db::{Request, Response},
    respond,
};
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::UnclaimedKits { member } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::reset::UnclaimedKit,
                    Response::UnclaimedKits,
                    "SELECT id,name FROM certifications c WHERE first_kit IS NOT NULL AND c.id NOT IN (SELECT cert FROM reset_kit WHERE MEMBER = $1) AND c.id IN (SELECT certification FROM certifications_trials WHERE trainee = $1 AND passed IS TRUE and (valid_until> NOW() or valid_until IS NULL))",
                    member.to_string(),
                )?;
                Ok(())
            }
            Self::CanClaim { member } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CanClaim,
                    "SELECT NOT EXISTS(SELECT 1 FROM reset_kit WHERE member = $1 AND created > NOW() - INTERVAL '13 days') as value",
                    member.to_string(),
                )
            }
            Self::ClaimKit { member, cert } => {
                sqlx::query!(
                    "INSERT INTO reset_kit (member, cert) VALUES ($1, $2)",
                    member.to_string(),
                    cert,
                )
                .execute(&*db)
                .await?;
                let Some(Value::Object(kit)) =
                    sqlx::query!("SELECT first_kit FROM certifications WHERE id = $1", cert)
                        .fetch_one(&*db)
                        .await
                        .map(|r| r.first_kit)?
                else {
                    return Ok(());
                };
                for item in kit {
                    if let Value::Number(n) = item.1 {
                        #[allow(clippy::cast_possible_truncation)]
                        sqlx::query!(
                            r#"
                                INSERT INTO
                                    gear_bank_purchases (member, class, quantity, personal, company)
                                VALUES
                                    ($1, $2, $3, 0, (SELECT SUM(company_current + personal_current) FROM gear_item_current_cost($2)))
                            "#,
                            member.to_string(),
                            item.0,
                            n.as_i64().unwrap_or(0) as i32,
                        )
                        .execute(&*db)
                        .await?;
                    }
                }
                respond!(msg, Response::ClaimKit(Ok(()))).await?;
                Ok(())
            }
        }
    }
}
