use async_trait::async_trait;
use serenity::model::prelude::{RoleId, UserId};
use synixe_events::certifications::{db::Response, publish::Publish};
use synixe_meta::discord::GUILD;
use synixe_proc::events_request;

use crate::cache_http::Bot;

use super::Listener;

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
                if trial.passed {
                    if let Ok(Ok((Response::List(Ok(certs)), _))) =
                        events_request!(nats, synixe_events::certifications::db, List {}).await
                    {
                        let Some(cert) = certs
                            .iter()
                            .find(|cert| cert.id == trial.certification) else {
                                warn!("Certification not found: {}", trial.certification);
                            return Ok(());
                        };
                        let mut member = GUILD
                            .member(Bot::get(), trial.trainee.parse::<UserId>()?)
                            .await?;
                        for role in &cert.roles_granted {
                            member
                                .add_role(&Bot::get().http, role.parse::<RoleId>()?)
                                .await?;
                        }
                        if let Err(e) = synixe_meta::discord::channel::TRAINING
                            .send_message(&*Bot::get(), |m| {
                                m.content(format!(
                                    "<@{}> has certified <@{}> in {}",
                                    trial.instructor, trial.trainee, cert.name
                                ))
                            })
                            .await
                        {
                            error!("Failed to send message: {}", e);
                        }
                    }
                } else {
                    let Ok(dm) = trial.trainee.parse::<UserId>().expect("Failed to parse user id").create_dm_channel(Bot::get()).await else {
                        warn!("Failed to create DM channel for {}", trial.trainee);
                        return Ok(());
                    };
                    if let Err(e) = dm.say(&*Bot::get(), format!("You failed your certification trial. Here are the notes from your instructor: \n > {}", trial.notes)).await {
                        error!("Failed to send message: {}", e);
                    }
                }
                Ok(())
            }
            Self::TrialExpiring { trial, days } => {
                if let Ok(Ok((Response::List(Ok(certs)), _))) =
                    events_request!(nats, synixe_events::certifications::db, List {}).await
                {
                    let Some(cert) = certs
                        .iter()
                        .find(|cert| cert.id == trial.certification) else {
                            warn!("Certification not found: {}", trial.certification);
                            return Ok(());
                    };
                    let message = if *days == 0 {
                        let mut member = GUILD
                            .member(Bot::get(), trial.trainee.parse::<UserId>()?)
                            .await?;
                        for role in &cert.roles_granted {
                            member
                                .remove_role(&Bot::get().http, role.parse::<RoleId>()?)
                                .await?;
                        }
                        format!(
                            "Your cert for {} has expired. Please contact an instructor to schedule a re-certification.",
                            cert.name,
                        )
                    } else {
                        format!(
                            "Your cert for {} expires in {} days. Please contact an instructor to schedule a re-certification.",
                            cert.name,
                            days,
                        )
                    };
                    let Ok(dm) = trial
                        .trainee
                        .parse::<UserId>()
                        .expect("Failed to parse user id")
                        .create_dm_channel(Bot::get())
                        .await else {
                            error!("Failed to create dm channel");
                            return Ok(());
                        };
                    if let Err(e) = dm.say(&*Bot::get(), message).await {
                        error!("Failed to send message: {}", e);
                    }
                }
                Ok(())
            }
        }
    }
}
