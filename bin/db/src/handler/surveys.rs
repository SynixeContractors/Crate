use async_trait::async_trait;
use synixe_events::{
    respond,
    surveys::db::{Request, Response},
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
            Self::SearchSurvey { title } => {
                if let Some(title) = title {
                    let resp = sqlx::query!(
                        r#"
                        SELECT id, title
                        FROM surveys
                        WHERE title ILIKE $1
                        AND (SELECT COUNT(*) FROM survey_options WHERE survey_id = surveys.id) != 0
                        "#,
                        format!("%{}%", title),
                    )
                    .fetch_all(&*db)
                    .await
                    .map_or_else(
                        |e| {
                            error!("Error searching for Survey: {}", e);
                            Response::SearchSurvey(Err(e.to_string()))
                        },
                        |rows| {
                            Response::SearchSurvey(Ok(rows
                                .into_iter()
                                .map(|row| (row.id, row.title))
                                .collect()))
                        },
                    );
                    let _ = respond!(msg, resp).await;
                } else {
                    let resp = sqlx::query!(
                        r#"
                        SELECT id, title
                        FROM surveys
                        WHERE id NOT IN (
                            SELECT survey_id FROM survey_entries GROUP BY survey_id
                        )
                        AND (SELECT COUNT(*) FROM survey_options WHERE survey_id = surveys.id) != 0
                        "#,
                    )
                    .fetch_all(&*db)
                    .await
                    .map_or_else(
                        |e| {
                            error!("Error getting Survey: {}", e);
                            Response::SearchSurvey(Err(e.to_string()))
                        },
                        |rows| {
                            Response::SearchSurvey(Ok(rows
                                .into_iter()
                                .map(|row| (row.id, row.title))
                                .collect()))
                        },
                    );
                    let _ = respond!(msg, resp).await;
                }
                Ok(())
            }
            Self::GetSurvey { survey } => {
                let resp = sqlx::query!(
                    r#"
                    SELECT title, description
                    FROM surveys
                    WHERE id = $1
                    "#,
                    survey,
                )
                .fetch_one(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error getting Survey: {}", e);
                        Response::GetSurvey(Err(e.to_string()))
                    },
                    |row| {
                        Response::GetSurvey(Ok(Some((
                            *survey,
                            row.title,
                            row.description.unwrap_or_default(),
                        ))))
                    },
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
            Self::GetOptions { survey } => {
                let resp = sqlx::query!(
                    r#"
                    SELECT options
                    FROM survey_options
                    WHERE survey_id = $1
                    "#,
                    survey,
                )
                .fetch_one(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error getting options: {}", e);
                        Response::GetOptions(Err(e.to_string()))
                    },
                    |row| Response::GetOptions(Ok(row.options)),
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
            Self::Submit {
                survey,
                member,
                option,
            } => {
                let resp = sqlx::query!(
                    r#"
                    INSERT INTO survey_entries (survey_id, member, option)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (survey_id, member)
                    DO UPDATE SET option = $3
                    "#,
                    survey,
                    member.to_string(),
                    option,
                )
                .execute(&*db)
                .await
                .map_or_else(
                    |e| {
                        error!("Error voting: {}", e);
                        Response::Submit(Err(e.to_string()))
                    },
                    |_| Response::Submit(Ok(())),
                );
                let _ = respond!(msg, resp).await;
                Ok(())
            }
        }
    }
}
