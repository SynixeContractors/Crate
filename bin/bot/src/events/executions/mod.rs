use nats::asynk::Message;
use synixe_events::{discord::executions, respond};

use crate::ArcCacheAndHttp;

mod postweeklytips;
mod updateactivityroles;

#[allow(clippy::too_many_lines)]
pub async fn handle(msg: Message, client: ArcCacheAndHttp) {
    let Ok((ev, _)) = synixe_events::parse_data!(msg, executions::Request) else {
        return;
    };
    match ev {
        executions::Request::UpdateActivityRoles {} => {
            if let Err(e) = respond!(msg, executions::Response::UpdateActivityRoles(Ok(()))).await {
                error!("Failed to respond to UpdateActivityRoles: {}", e);
            }
            updateactivityroles::execute(client).await;
        }
        executions::Request::PostWeeklyTips {} => {
            if let Err(e) = respond!(msg, executions::Response::PostWeeklyTips(Ok(()))).await {
                error!("Failed to respond to PostWeeklyTips: {}", e);
            }
            postweeklytips::execute(client).await;
        }
    }
}
