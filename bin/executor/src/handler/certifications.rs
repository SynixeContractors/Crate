use async_trait::async_trait;
use serenity::model::prelude::{RoleId, UserId};
use synixe_events::{
    certifications::{
        db,
        executions::{Request, Response},
    },
    publish, respond,
};
use synixe_proc::events_request_5;
use time::OffsetDateTime;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::CheckExpiries {} => {
                respond!(msg, Response::CheckExpiries(Ok(()))).await?;
                let Ok(Ok((db::Response::AllExpiring(Ok(expiring)), _))) = events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    AllExpiring { days: 30 }
                )
                .await
                else {
                    return Ok(());
                };
                for trial in expiring {
                    if let Some(valid_until) = trial.valid_until {
                        let diff = valid_until - OffsetDateTime::now_utc();
                        if diff.whole_days() == 29 {
                            let nats = bootstrap::NC::get().await;
                            publish!(
                                nats,
                                synixe_events::certifications::publish::Publish::TrialExpiring {
                                    trial: trial.clone(),
                                    days: 30,
                                }
                            )
                            .await?;
                        }
                        if diff.whole_days() == 14 {
                            let nats = bootstrap::NC::get().await;
                            publish!(
                                nats,
                                synixe_events::certifications::publish::Publish::TrialExpiring {
                                    trial: trial.clone(),
                                    days: 15,
                                }
                            )
                            .await?;
                        }
                        if diff.whole_days() == 0 {
                            let nats = bootstrap::NC::get().await;
                            publish!(
                                nats,
                                synixe_events::certifications::publish::Publish::TrialExpiring {
                                    trial: trial.clone(),
                                    days: 0,
                                }
                            )
                            .await?;
                        }
                    }
                }
                Ok(())
            }
            Self::CheckRoles {} => {
                respond!(msg, Response::CheckRoles(Ok(()))).await?;
                let Ok(Ok((db::Response::AllActive(Ok(active)), _))) = events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    AllActive {}
                )
                .await
                else {
                    return Ok(());
                };
                if let Ok(Ok((db::Response::List(Ok(certs)), _))) = events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    List {}
                )
                .await
                {
                    for trial in active {
                        let cert = certs
                            .iter()
                            .find(|cert| cert.id == trial.certification)
                            .expect("Certification not found");
                        if let Err(e) = events_request_5!(
                            bootstrap::NC::get().await,
                            synixe_events::discord::write,
                            EnsureRoles {
                                member: UserId::new(
                                    trial.trainee.parse().expect("Invalid user id")
                                ),
                                roles: cert
                                    .roles_granted
                                    .iter()
                                    .map(|r| RoleId::new(r.parse().expect("Invalid role id")))
                                    .collect(),
                            }
                        )
                        .await
                        .expect("Failed to ensure roles")
                        {
                            error!("Failed to ensure roles for trial {}: {}", trial.id, e);
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
