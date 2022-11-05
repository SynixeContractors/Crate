use async_trait::async_trait;
use serenity::model::prelude::UserId;
use synixe_events::certifications::{db::Response, publish::Publish};
use synixe_proc::events_request;

use crate::cache_http::Bot;

use super::Listener;

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::TrialSubmitted { trial } => {
                info!("Trial submitted: {:?}", trial);
                if trial.passed {
                    if let Ok(((Response::List(Ok(certs)), _), _)) =
                        events_request!(nats, synixe_events::certifications::db, List {}).await
                    {
                        synixe_meta::discord::channel::TRAINING
                            .send_message(&*Bot::get(), |m| {
                                m.content(format!(
                                    "<@{}> has certified <@{}> in {}",
                                    trial.instructor,
                                    trial.trainee,
                                    certs
                                        .iter()
                                        .find(|cert| cert.id == trial.certification)
                                        .unwrap()
                                        .name
                                ))
                            })
                            .await
                            .unwrap();
                    }
                } else {
                    trial.trainee.parse::<UserId>().unwrap().create_dm_channel(Bot::get()).await.unwrap().say(&*Bot::get(), format!("You failed your certification trial. Here are the notes from your instructor: \n > {}", trial.notes)).await.unwrap();
                }
                Ok(())
            }
        }
    }
}
