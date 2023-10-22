use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_meta::discord::BRODSKY;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::RUNTIME;

pub fn group() -> Group {
    Group::new().command("auto", auto)
}

fn auto(discord: String, certification: Uuid, passed: bool) {
    let Ok(discord) = discord.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        debug!("certification auto for {}", discord);
        let _ = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::certifications::db,
            Certify {
                instructor: BRODSKY,
                trainee: UserId(discord),
                certification,
                notes: "Automated certification".to_string(),
                passed
            }
        )
        .await;
    });
}
