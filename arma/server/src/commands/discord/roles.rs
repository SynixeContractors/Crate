use arma_rs::Group;
use synixe_proc::events_request_5;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("get", command_get)
}

fn command_get() {
    RUNTIME.spawn(async move {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("command received before context was initialized");
            return;
        };
        let Ok(Ok((synixe_events::discord::info::Response::AllRoles(Ok(roles)), _))) =
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::discord::info,
                AllRoles {}
            )
            .await
        else {
            error!("failed to fetch discord roles over nats");
            return;
        };
        if let Err(e) = context.callback_data(
            "crate:discord",
            "roles:get:ok",
            roles
                .into_iter()
                .map(|(id, name)| {
                    arma_rs::Value::Array(vec![
                        arma_rs::Value::String(id.to_string()),
                        arma_rs::Value::String(name),
                    ])
                })
                .collect::<Vec<_>>(),
        ) {
            error!("error sending member:get:ok: {e:?}");
        }
    });
}
