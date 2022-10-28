#![allow(clippy::cast_possible_wrap)]

use async_trait::async_trait;
use sqlx::types::Json;
use synixe_events::{
    certifications::db::{Request, Response},
    respond,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::List {} => fetch_as_and_respond!(
                msg,
                *db,
                cx,
                synixe_model::certifications::Certification,
                Response::List,
                r#"
                    SELECT
                        id,
                        name,
                        link,
                        roles_required as "roles_required: Json<synixe_model::Roles>",
                        roles_granted as "roles_granted: Json<synixe_model::Roles>",
                        valid_for,
                        created_at
                    FROM
                        certifications"#,
            ),
            Self::Certify {
                instructor,
                trainee,
                certification,
                notes,
                passed,
            } => {
                // Check if the instructor is certified to teach the certification
                let instructor_certified = sqlx::query!(
                    r#"
                        SELECT
                            EXISTS(
                                SELECT
                                    1
                                FROM
                                    certifications_instructors
                                WHERE
                                    user_id = $1
                                    AND certification_id = $2
                            ) AS "instructor_certified!""#,
                    instructor.0 as i64,
                    certification.as_bytes() as _,
                )
                .fetch_one(&*db)
                .await?
                .instructor_certified;
                if !instructor_certified {
                    if let Err(e) = respond!(
                        msg,
                        Response::Certify(Err(
                            "Instructor is not certified to teach this certification".to_string()
                        ))
                    )
                    .await
                    {
                        error!("Failed to respond to message: {}", e);
                    }
                    return Ok(());
                }
                // Get the days the certification is valid for
                let valid_for = sqlx::query!(
                    r#"
                        SELECT
                            valid_for
                        FROM
                            certifications
                        WHERE
                            id = $1"#,
                    certification.as_bytes() as _,
                )
                .fetch_one(&*db)
                .await?
                .valid_for;
                // Create the certification trial, returning it
                if *passed {
                    fetch_one_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::certifications::CertificationTrial,
                        Response::Certify,
                        r#"
                            INSERT INTO
                                certifications_trials
                                (instructor_id, trainee_id, certification_id, notes, valid_until)
                            VALUES
                                ($1, $2, $3, $4, NOW() + ($5 || ' days')::INTERVAL)
                            RETURNING
                                id,
                                instructor_id,
                                trainee_id,
                                certification_id,
                                notes,
                                valid_until,
                                created_at"#,
                        instructor.0 as i64,
                        trainee.0 as i64,
                        certification,
                        notes,
                        valid_for.to_string(),
                    )
                } else {
                    fetch_one_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::certifications::CertificationTrial,
                        Response::Certify,
                        r#"
                            INSERT INTO
                                certifications_trials
                                (instructor_id, trainee_id, certification_id, notes)
                            VALUES
                                ($1, $2, $3, $4)
                            RETURNING
                                id,
                                instructor_id,
                                trainee_id,
                                certification_id,
                                notes,
                                valid_until,
                                created_at"#,
                        instructor.0 as i64,
                        trainee.0 as i64,
                        certification,
                        notes,
                    )
                }
            }
            Self::Active { user } => todo!(),
        }
    }
}
