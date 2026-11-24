#![allow(clippy::cast_possible_wrap)]

use async_trait::async_trait;
use synixe_events::{
    casino::db::{GameStats, Request, Response},
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
            Self::History { member } => {
                let result = sqlx::query!(
                    r#"SELECT
                        split_part(reason, ': ', 2) AS "game!",
                        COUNT(*) FILTER (WHERE reason LIKE 'casino buy:%') AS "bets!",
                        COALESCE(-SUM(amount) FILTER (WHERE reason LIKE 'casino buy:%'), 0) AS "wagered!",
                        COALESCE(SUM(amount) FILTER (WHERE reason LIKE 'casino cashout:%'), 0) AS "paid_out!",
                        COUNT(*) FILTER (WHERE reason LIKE 'casino cashout:%') AS "wins!",
                        COALESCE(MAX(amount) FILTER (WHERE reason LIKE 'casino cashout:%'), 0)::BIGINT AS "biggest_win!"
                    FROM gear_bank_deposits
                    WHERE member = $1 AND id = '00000000-0000-0000-0000-000000000010'
                    GROUP BY 1
                    ORDER BY 2 DESC"#,
                    member.to_string(),
                )
                .fetch_all(&*db)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| GameStats {
                            game: row.game,
                            bets: row.bets,
                            wagered: row.wagered,
                            paid_out: row.paid_out,
                            wins: row.wins,
                            biggest_win: row.biggest_win,
                        })
                        .collect()
                })
                .map_err(|e| {
                    error!("failed to fetch casino history: {e}");
                    e.to_string()
                });
                respond!(msg, Response::History(result)).await?;
            }
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
            }
        }
        Ok(())
    }
}
