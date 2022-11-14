use arma_rs::{Group, IntoArma};
use synixe_events::gear::db;
use synixe_proc::events_request;

use crate::{CONTEXT, RUNTIME};

pub fn group() -> Group {
    Group::new().command("items", command_items)
}

fn command_items() {
    RUNTIME.spawn(async {
        let Ok(((db::Response::ShopGetAll(Ok(items)), _), _)) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            ShopGetAll {}
        ).await else {
            error!("failed to fetch shop items over nats");
            return;
        };
        CONTEXT
            .read()
            .await
            .as_ref()
            .unwrap()
            .callback_null("crate:gear:shop", "items:clear");
        for (class, (roles, price)) in items {
            CONTEXT.read().await.as_ref().unwrap().callback_data(
                "crate:gear:shop",
                "items:add",
                vec![
                    class.to_arma(),
                    arma_rs::Value::Array(vec![
                        roles.unwrap_or_default().to_arma(),
                        price.to_arma(),
                    ]),
                ],
            );
        }
        CONTEXT
            .read()
            .await
            .as_ref()
            .unwrap()
            .callback_null("crate:gear:shop", "items:done");
    });
}
