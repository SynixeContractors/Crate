use async_trait::async_trait;
use synixe_events::{
    respond,
    voting::db::{Request, Response},
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::CheckTicket { ticket } => {
                let resp = sqlx::query!(
                    r#"
                    SELECT EXISTS (
                        SELECT 1
                        FROM voting_ticket_box
                        WHERE encrypted_ticket = $1
                    )
                    "#,
                    ticket,
                )
                .fetch_one(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error checking ticket: {}", e);
                        Response::CheckTicket(Err(e.to_string()))
                    },
                    |row| Response::CheckTicket(Ok(row.exists.unwrap_or(false))),
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
            Self::GetPoll { poll } => {
                let resp = sqlx::query!(
                    r#"
                    SELECT title, description, public_key
                    FROM voting_polls
                    WHERE id = $1
                    "#,
                    poll,
                )
                .fetch_one(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error getting poll: {}", e);
                        Response::GetPoll(Err(e.to_string()))
                    },
                    |row| {
                        Response::GetPoll(Ok(Some((
                            *poll,
                            row.title,
                            row.description,
                            row.public_key,
                        ))))
                    },
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
            Self::GetOptions { poll } => {
                let resp = sqlx::query!(
                    r#"
                    SELECT id, title
                    FROM voting_options
                    WHERE poll_id = $1
                    "#,
                    poll,
                )
                .fetch_all(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error getting options: {}", e);
                        Response::GetOptions(Err(e.to_string()))
                    },
                    |rows| {
                        let options = rows
                            .into_iter()
                            .map(|row| (row.id, row.title))
                            .collect::<Vec<(uuid::Uuid, String)>>();
                        Response::GetOptions(Ok(options))
                    },
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
            Self::Vote {
                poll,
                ticket,
                option,
            } => {
                if let Err(e) = sqlx::query!(
                    r#"
                    INSERT INTO voting_ticket_box (poll_id, encrypted_ticket)
                    VALUES ($1, $2)
                    ON CONFLICT (poll_id, encrypted_ticket) DO NOTHING
                    "#,
                    poll,
                    ticket.clone(),
                )
                .execute(&*db)
                .await
                {
                    error!("Error inserting ticket: {}", e);
                    let resp = Response::Vote(Err(e.to_string()));
                    let _ = respond!(msg, resp).await;
                    return Ok(());
                }
                if let Err(e) = sqlx::query!(
                    r#"
                    INSERT INTO voting_vote_box (poll_id, encrypted_vote)
                    VALUES ($1, $2)
                    ON CONFLICT (poll_id, encrypted_vote) DO NOTHING
                    "#,
                    poll,
                    option,
                )
                .execute(&*db)
                .await
                {
                    error!("Error inserting vote: {}", e);
                    let resp = Response::Vote(Err(e.to_string()));
                    let _ = respond!(msg, resp).await;
                    return Ok(());
                }
                let _ = respond!(msg, Response::Vote(Ok(()))).await;
                Ok(())
            }
        }
    }
}
