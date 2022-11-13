use std::time::Duration;

use synixe_events::{arma_server::publish::Publish, publish};

use crate::{listener::Listener, SERVER_ID};

pub async fn heart() {
    loop {
        tokio::time::sleep(Duration::from_secs(15)).await;
        // ctx.callback_null("crate_server", "beat");
        if let Err(e) = publish!(
            bootstrap::NC::get().await,
            Publish::Heartbeat {
                id: SERVER_ID.clone(),
            }
        )
        .await
        {
            error!("failed to publish heartbeat: {}", e);
        }
    }
}

pub async fn events() {
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe("synixe.publish.>", &format!("arma-server-{}", *SERVER_ID))
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::listener!(msg, nats, synixe_events::discord::publish::Publish);
    }
}
