#![allow(clippy::cast_possible_wrap)]

use async_trait::async_trait;
use synixe_events::{
    certifications::db::{Request, Response},
    publish, respond,
};

use crate::handler::reputation::audit;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
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
                            array_agg(ci.member) as instructors,
                            name,
                            link,
                            roles_required,
                            roles_granted,
                            valid_for,
                            c.created
                        FROM
                            certifications c
                        INNER JOIN
                            certifications_instructors ci
                        ON
                            c.id = ci.certification
                        GROUP BY
                            c.id;"#,
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
                            id,
                            array_agg(ci2.member) as instructors,
                            name,
                            link,
                            roles_required,
                            roles_granted,
                            valid_for,
                            c.created
                        FROM
                            certifications c
                        INNER JOIN
                            certifications_instructors ci
                        ON
                            ci.certification = c.id
                        INNER JOIN certifications_instructors ci2
                            ON ci2.certification = c.id
                        WHERE
                            ci.member = $1
                        GROUP BY c.id;"#,
                    member.to_string(),
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
                    instructor.to_string(),
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
                        cx.clone(),
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
                        instructor.to_string(),
                        trainee.to_string(),
                        certification,
                        notes,
                        valid_for,
                    )?
                } else {
                    fetch_one_as_and_respond!(
                        msg,
                        *db,
                        cx.clone(),
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
                        instructor.to_string(),
                        trainee.to_string(),
                        certification,
                        notes,
                    )?
                };
                publish!(
                    nats,
                    synixe_events::certifications::publish::Publish::TrialSubmitted { trial }
                )
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
                        DISTINCT ON (certification)
                        certification,
                        id,
                        instructor,
                        trainee,
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
                    ORDER BY certification, created DESC"#,
                    member.to_string(),
                )?;
                Ok(())
            }
            Self::AllActive {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::certifications::CertificationTrial,
                    Response::AllActive,
                    r#"
                        SELECT
                            DISTINCT ON (trainee, certification)
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
                            passed IS TRUE
                            AND (valid_until > NOW() OR valid_until IS NULL)
                        ORDER BY trainee, certification, created DESC"#,
                )?;
                Ok(())
            }
            Self::AllExpiring { days } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::certifications::CertificationTrial,
                    Response::AllExpiring,
                    r#"
                        SELECT
                            DISTINCT ON (trainee, certification)
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
                            certifications_trials as ct
                        WHERE
                            passed IS TRUE
                            AND (valid_until > NOW())
                            AND valid_until < NOW() + $1 * INTERVAL '1 day'
                            AND NOT EXISTS (
                                SELECT 1 FROM certifications_trials as ict WHERE
                                    ct.certification = ict.certification
                                    AND ct.trainee = ict.trainee
                                    AND valid_until > NOW() + $1 * INTERVAL '1 day')
                        ORDER BY trainee, certification, created DESC"#,
                    f64::from(*days),
                )?;
                Ok(())
            }
            Self::PassedCount {
                certification,
                member,
            } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::PassedCount,
                    r#"
                        SELECT
                            COUNT(*) AS "value!"
                        FROM
                            certifications_trials
                        WHERE
                            certification = $1
                            AND trainee = $2
                            AND passed IS TRUE"#,
                    certification,
                    member.to_string(),
                )
            }
            Self::FirstKits { certification } => {
                if let Some(certification) = certification {
                    fetch_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::certifications::CertificationFirstKit,
                        Response::FirstKits,
                        r#"
                            SELECT
                                id,
                                certification,
                                name,
                                description,
                                first_kit
                            FROM
                                certifications_first_kit
                            WHERE
                                (certification = $1)"#,
                        certification,
                    )?;
                } else {
                    fetch_as_and_respond!(
                        msg,
                        *db,
                        cx,
                        synixe_model::certifications::CertificationFirstKit,
                        Response::FirstKits,
                        r#"
                            SELECT
                                id,
                                certification,
                                name,
                                description,
                                first_kit
                            FROM
                                certifications_first_kit"#,
                    )?;
                }
                Ok(())
            }
            Self::GiveFirstKit { first_kit, member } => {
                let message = audit(format!(
                    "Giving first kit {} to member <@{}>",
                    first_kit, member
                ))
                .await;
                sqlx::query!(
                    r#"
                        SELECT
                            give_first_kit($1, $2)
                    "#,
                    member.to_string(),
                    first_kit,
                )
                .execute(&*db)
                .await?;
                respond!(msg, Response::GiveFirstKit(Ok(()))).await?;
                Ok(())
            }
        }
    }
}
