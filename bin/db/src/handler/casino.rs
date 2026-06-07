#![allow(clippy::cast_possible_wrap)]

use async_trait::async_trait;
use synixe_events::{
    casino::db::{Request, Response},
    respond,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: async_nats::message::Message,
        _nats: async_nats::Client,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::BuyIn {
                member,
                game,
                amount,
            } => {
                sqlx::query!(
                    r#"
                        INSERT INTO
                            gear_bank_deposits
                            (member, amount, reason, id)
                        VALUES
                            ($1, $2, $3, '00000000-0000-0000-0000-000000000010')
                        ON CONFLICT DO NOTHING"#,
                    member.to_string(),
                    -*amount,
                    format!("casino buy: {}", game)
                )
                .execute(&*db)
                .await?;
                respond!(msg, Response::BuyIn(Ok(()))).await?;
                Ok(())
            }
            Self::CashOut {
                member,
                game,
                amount,
            } => {
                sqlx::query!(
                    r#"
                        INSERT INTO
                            gear_bank_deposits
                            (member, amount, reason, id)
                        VALUES
                            ($1, $2, $3, '00000000-0000-0000-0000-000000000010')
                        ON CONFLICT DO NOTHING"#,
                    member.to_string(),
                    amount,
                    format!("casino cashout: {}", game)
                )
                .execute(&*db)
                .await?;
                respond!(msg, Response::CashOut(Ok(()))).await?;
                Ok(())
            }
        }
    }
}
