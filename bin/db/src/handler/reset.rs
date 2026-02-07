use super::Handler;
use async_trait::async_trait;
use serde_json::Value;
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
                    "SELECT certification as id,name,specialist FROM certifications_first_kit c WHERE c.certification NOT IN (SELECT cert FROM reset_kit WHERE MEMBER = $1) AND c.certification IN (SELECT certification FROM certifications_trials WHERE trainee = $1 AND passed IS TRUE and (valid_until> NOW() or valid_until IS NULL))",
                    member.to_string(),
                )?;
                Ok(())
            }
            Self::CanClaim { member, cert } => {
                // Check if the cert is a specialist cert, is not, always return true
                let is_specialist = sqlx::query!(
                    "SELECT specialist FROM certifications_first_kit WHERE certification = $1",
                    cert
                )
                .fetch_one(&*db)
                .await?
                .specialist;
                if !is_specialist {
                    respond!(msg, Response::CanClaim(Ok(Some(Some(true))))).await?;
                    return Ok(());
                }
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::CanClaim,
                    "SELECT NOT EXISTS(SELECT 1 FROM reset_kit WHERE member = $1 AND created > NOW() - INTERVAL '13 days') as value",
                    member.to_string(),
                )
            }
            Self::LastClaim { member } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::LastClaim,
                    "SELECT created as value FROM reset_kit WHERE member = $1 ORDER BY created DESC LIMIT 1",
                    member.to_string(),
                )
            }
            Self::ClaimKit { member, cert } => {
                let (Value::Object(kit), specialist) =
                    sqlx::query!("SELECT first_kit, specialist FROM certifications_first_kit WHERE certification = $1", cert)
                        .fetch_one(&*db)
                        .await
                        .map(|r| (r.first_kit, r.specialist))?
                else {
                    return Ok(());
                };
                sqlx::query!(
                    "INSERT INTO reset_kit (member, cert, specialist) VALUES ($1, $2, $3)",
                    member.to_string(),
                    cert,
                    specialist,
                )
                .execute(&*db)
                .await?;
                for item in kit {
                    if let Value::Number(n) = item.1 {
                        #[allow(clippy::cast_possible_truncation)]
                        sqlx::query!(
                            r#"
                                INSERT INTO
                                    gear_bank_purchases (reason, member, class, quantity, personal, company)
                                VALUES
                                    ('kit reset', $1, $2, $3, 0, (SELECT SUM(company_current + personal_current) FROM gear_item_current_cost($2)))
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
