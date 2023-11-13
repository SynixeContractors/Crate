use serenity::{
    model::prelude::application_command::ApplicationCommandInteraction, prelude::Context,
};

use serenity::model::application::interaction::application_command::CommandDataOption;
use synixe_events::garage::db::{Response, ShopOrder};
use synixe_meta::discord::role::LEADERSHIP;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::discord::slash::ShouldAsk;
use crate::discord::utils::audit;
use crate::discord::{self, interaction::Interaction};
use crate::get_option;

#[allow(clippy::too_many_lines)]
pub async fn purchase(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(
        ctx,
        discord::interaction::Generic::Application(command),
        options,
    );
    super::super::requires_roles(
        command.user.id,
        &[LEADERSHIP],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("garage purchase", options)),
        &mut interaction,
    )
    .await?;

    let Some(plate) = get_option!(options, "plate", String) else {
        return interaction
            .reply("Required option not provided: plate")
            .await;
    };

    let kind = options
        .iter()
        .rev()
        .find(|option| option.name == "vehicle" || option.name == "addon");

    let Some(kind) = kind else {
        return interaction
            .reply("Required option not provided: vehicle or addon")
            .await;
    };

    let Some(id) = get_option!(options, &kind.name, String) else {
        return interaction
            .reply("Required option not provided: vehicle or addon")
            .await;
    };
    let Ok(id) = Uuid::parse_str(id.as_str()) else {
        return interaction.reply("Invalid vehicle or addon UUID").await;
    };

    match kind.name.as_str() {
        "vehicle" => {
            let Ok(Ok((Response::PurchaseShopAsset(Ok(())), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::garage::db,
                PurchaseShopAsset {
                    order: ShopOrder::Vehicle {
                        id,
                        plate: Some(plate.clone()),
                        color: get_option!(options, "color", String).cloned(),
                        member: command
                            .member
                            .as_ref()
                            .expect("member should always exist on guild commands")
                            .user
                            .id
                    }
                }
            )
            .await
            else {
                return interaction.reply("Error purchasing vehicle").await;
            };
            audit(format!(
                "Vehicle `{}` purchased by <@{}>",
                plate, command.user.id,
            ));
            interaction.reply("**Vehicle Purchased**\n\n").await
        }
        "addon" => {
            let Ok(Ok((Response::PurchaseShopAsset(Ok(())), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::garage::db,
                PurchaseShopAsset {
                    order: ShopOrder::Addon {
                        id,
                        member: command
                            .member
                            .as_ref()
                            .expect("member should always exist on guild commands")
                            .user
                            .id
                    }
                }
            )
            .await
            else {
                return interaction.reply("Error purchasing addon").await;
            };
            audit(format!(
                "Addon `{}` purchased for `{}` by <@{}>",
                id, plate, command.user.id,
            ));
            interaction.reply("**Addon Purchased**\n\n").await
        }
        _ => {
            interaction
                .reply("Invalid option provided: vehicle or addon")
                .await
        }
    }
}
