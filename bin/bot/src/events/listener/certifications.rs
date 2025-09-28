use async_trait::async_trait;
use serenity::model::prelude::{RoleId, UserId};
use synixe_events::certifications::{db::Response, publish::Publish};
use synixe_meta::discord::GUILD;
use synixe_proc::events_request_5;

use crate::cache_http::CacheAndHttp;

use super::Listener;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::TrialSubmitted { trial } => {
                info!("Trial submitted: {:?}", trial);
                if trial.passed
                    && let Ok(Ok((Response::List(Ok(certs)), _))) =
                        events_request_5!(nats, synixe_events::certifications::db, List {}).await
                {
                    let Some(cert) = certs.iter().find(|cert| cert.id == trial.certification)
                    else {
                        warn!("Certification not found: {}", trial.certification);
                        return Ok(());
                    };
                    let member = GUILD
                        .member(
                            CacheAndHttp::get().as_ref(),
                            trial.trainee.parse::<UserId>()?,
                        )
                        .await?;
                    for role in &cert.roles_granted {
                        member
                            .add_role(CacheAndHttp::get().as_ref(), role.parse::<RoleId>()?)
                            .await?;
                    }
                    if let Err(e) = synixe_meta::discord::channel::TRAINING
                        .say(
                            CacheAndHttp::get().as_ref(),
                            format!(
                                "<@{}> has certified <@{}> in {}",
                                trial.instructor, trial.trainee, cert.name
                            ),
                        )
                        .await
                    {
                        error!("Failed to send message: {}", e);
                    }
                }
                Ok(())
            }
            Self::TrialExpiring { trial, days } => {
                if let Ok(Ok((Response::List(Ok(certs)), _))) =
                    events_request_5!(nats, synixe_events::certifications::db, List {}).await
                {
                    let Some(cert) = certs.iter().find(|cert| cert.id == trial.certification)
                    else {
                        warn!("Certification not found: {}", trial.certification);
                        return Ok(());
                    };

                    let Ok(member) = GUILD
                        .member(
                            CacheAndHttp::get().as_ref(),
                            trial.trainee.parse::<UserId>()?,
                        )
                        .await
                    else {
                        warn!("Failed to get member: {}", trial.trainee);
                        return Ok(());
                    };

                    let message = if *days == 0 {
                        for role in &cert.roles_granted {
                            member
                                .remove_role(&CacheAndHttp::get().as_ref(), role.parse::<RoleId>()?)
                                .await?;
                        }
                        format!(
                            "Your cert for {} has expired. Please contact an instructor to schedule a re-certification.",
                            cert.name,
                        )
                    } else {
                        format!(
                            "Your cert for {} expires in {} days. Please contact an instructor to schedule a re-certification.",
                            cert.name, days,
                        )
                    };
                    let Ok(dm) = trial
                        .trainee
                        .parse::<UserId>()
                        .expect("Failed to parse user id")
                        .create_dm_channel(CacheAndHttp::get().as_ref())
                        .await
                    else {
                        error!("Failed to create dm channel");
                        return Ok(());
                    };
                    if let Err(e) = dm.say(CacheAndHttp::get().as_ref(), &message).await {
                        error!("Failed to send message: {}", e);
                    }
                    if let Err(e) = synixe_meta::discord::channel::LOG
                        .say(
                            CacheAndHttp::get().as_ref(),
                            format!("<@{}> has been notified\n> {message}", trial.trainee,),
                        )
                        .await
                    {
                        error!("Cannot send ban message: {}", e);
                    }
                }
                Ok(())
            }
        }
    }
}
