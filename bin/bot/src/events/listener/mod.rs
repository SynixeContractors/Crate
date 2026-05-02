use synixe_events::global::Publish;

include!("../../../../../lib/common/listener.rs");

mod certifications;
mod gear;
mod missions;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let mut sub = nats
        .queue_subscribe("synixe.publish.>", String::from("synixe-bot"))
        .await
        .expect("Failed to subscribe to synixe.publish.*");
    while let Some(msg) = sub.next().await {
        synixe_events::listener!(
            msg.clone(),
            nats.clone(),
            synixe_events::certifications::publish::Publish,
            synixe_events::gear::publish::Publish,
            synixe_events::missions::publish::Publish,
            synixe_events::global::Publish,
        );
    }
}

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: async_nats::message::Message,
        nats: async_nats::Client,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::Tick { time } => {
                info!("Tick: {:?}", time);
                if time.minute() % 5 == 0 {
                    missions::tick(nats).await;
                }
            }
        }
        Ok(())
    }
}
