#![allow(clippy::cast_possible_wrap)]

use async_trait::async_trait;
use opentelemetry::trace::FutureExt;
use synixe_events::{
    certifications::db::{Request, Response},
    publish, respond,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
        cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::List {} => {
                fetch_as_and_respond!(
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
                            roles_required,
                            roles_granted,
                            valid_for,
                            created
                        FROM
                            certifications"#,
                )?;
                Ok(())
            }
            Self::ListInstructor { member } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::certifications::Certification,
                    Response::ListInstructor,
                    r#"
                        SELECT
                            c.id,
                            c.name,
                            c.link,
                            c.roles_required,
                            c.roles_granted,
                            c.valid_for,
                            c.created
                        FROM
                            certifications c
                        INNER JOIN
                            certifications_instructors ci
                        ON
                            ci.certification = c.id
                        WHERE
                            ci.member = $1"#,
                    member.0.to_string(),
                )?;
                Ok(())
            }
            Self::Name { certification } => fetch_one_and_respond!(
                msg,
                *db,
                cx,
                Response::Name,
                r#"
                    SELECT
                        name AS value
                    FROM
                        certifications
                    WHERE
                        id = $1"#,
                certification,
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
                                    member = $1
                                    AND certification = $2
                            ) AS "instructor_certified!""#,
                    instructor.0.to_string(),
                    certification,
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
                    certification,
                )
                .fetch_one(&*db)
                .await?
                .valid_for;
                // Create the certification trial, returning it
                let trial = if *passed {
                    fetch_one_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::certifications::CertificationTrial,
                        Response::Certify,
                        r#"
                            INSERT INTO
                                certifications_trials
                                (instructor, trainee, certification, notes, passed, valid_for)
                            VALUES
                                ($1, $2, $3, $4, true, $5)
                            RETURNING
                                id,
                                instructor,
                                trainee,
                                certification,
                                notes,
                                passed,
                                valid_for,
                                valid_until,
                                created"#,
                        instructor.0.to_string(),
                        trainee.0.to_string(),
                        certification,
                        notes,
                        valid_for,
                    )?
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
                                (instructor, trainee, certification, notes, passed)
                            VALUES
                                ($1, $2, $3, $4, false)
                            RETURNING
                                id,
                                instructor,
                                trainee,
                                certification,
                                notes,
                                passed,
                                valid_for,
                                valid_until,
                                created"#,
                        instructor.0.to_string(),
                        trainee.0.to_string(),
                        certification,
                        notes,
                    )?
                };
                publish!(
                    nats,
                    synixe_events::certifications::publish::Publish::TrialSubmitted { trial }
                )
                .with_context(cx)
                .await?;
                Ok(())
            }
            Self::Active { member } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::certifications::CertificationTrial,
                    Response::Active,
                    r#"
                        SELECT
                            id,
                            instructor,
                            trainee,
                            certification,
                            notes,
                            passed,
                            valid_for,
                            valid_until,
                            created
                        FROM
                            certifications_trials
                        WHERE
                            trainee = $1
                            AND passed IS TRUE
                            AND (valid_until > NOW() OR valid_until IS NULL)
                        GROUP BY id ORDER BY created DESC"#,
                    member.0.to_string(),
                )?;
                Ok(())
            }
        }
    }
}
