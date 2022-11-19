use async_trait::async_trait;
use synixe_events::{
    certifications::{
        db,
        executions::{Request, Response},
    },
    publish, respond,
};
use synixe_proc::events_request;
use time::OffsetDateTime;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::CheckExpiries {} => {
                respond!(msg, Response::CheckExpiries(Ok(()))).await?;
                let Ok(((db::Response::Expiring(Ok(expiring)), _), _)) = events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    Expiring { days: 30 }
                ).await else {
                    return Ok(());
                };
                for trial in expiring {
                    if let Some(valid_until) = trial.valid_until {
                        let diff = valid_until - OffsetDateTime::now_utc();
                        println!("{}: {}", trial.trainee, diff.whole_days());
                        if diff.whole_days() == 29 {
                            publish!(
                                bootstrap::NC::get().await,
                                synixe_events::certifications::publish::Publish::TrialExpiring {
                                    trial: trial.clone(),
                                    days: 30,
                                }
                            )
                            .await?;
                        }
                        if diff.whole_days() == 14 {
                            publish!(
                                bootstrap::NC::get().await,
                                synixe_events::certifications::publish::Publish::TrialExpiring {
                                    trial: trial.clone(),
                                    days: 15,
                                }
                            )
                            .await?;
                        }
                        if diff.whole_days() == -1 {
                            publish!(
                                bootstrap::NC::get().await,
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
        }
    }
}
