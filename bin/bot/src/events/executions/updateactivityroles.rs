use serenity::{all::UserId, futures::StreamExt};
use synixe_events::discord::db::Response;
use synixe_meta::discord::{GUILD, channel::LOG, role::ACTIVE};
use synixe_proc::events_request_2;

use crate::ArcCacheAndHttp;

#[allow(clippy::cognitive_complexity)]
pub async fn execute(client: ArcCacheAndHttp) {
    let Ok(Ok((Response::ActiveMembers(Ok(active)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::discord::db,
        ActiveMembers {}
    )
    .await
    else {
        error!("Failed to get locker balance");
        return;
    };
    let active = active
        .into_iter()
        .filter(|x| x != "0")
        .map(|x| UserId::new(x.parse::<u64>().expect("Failed to parse UserId")))
        .collect::<Vec<_>>();
    let mut members = GUILD.members_iter(client.as_ref()).boxed();
    while let Some(Ok(member)) = members.next().await {
        if active.contains(&member.user.id) && !member.roles.contains(&ACTIVE) {
            if let Err(e) = member.add_role(client.as_ref(), ACTIVE).await {
                error!("Failed to add role to {}: {}", member.user.id, e);
            } else {
                let _ = LOG
                    .say(
                        client.as_ref(),
                        format!("<@{}> is now active", member.user.id),
                    )
                    .await;
            }
            continue;
        }
        if !active.contains(&member.user.id) && member.roles.contains(&ACTIVE) {
            if let Err(e) = member.remove_role(client.as_ref(), ACTIVE).await {
                error!("Failed to remove role from {}: {}", member.user.id, e);
            } else {
                let _ = LOG
                    .say(
                        client.as_ref(),
                        format!("<@{}> is no longer active", member.user.id),
                    )
                    .await;
            }
        }
    }
}
