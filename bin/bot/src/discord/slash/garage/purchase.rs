use serenity::all::{CommandDataOption, CommandInteraction};
use serenity::client::Context;
use synixe_events::garage::db::{Response, ShopOrder};
use synixe_meta::discord::role::GARAGE;
use synixe_proc::events_request_2;
use uuid::Uuid;

use crate::discord::interaction::Interaction;
use crate::discord::slash::ShouldAsk;
use crate::{audit, get_option};

#[allow(clippy::too_many_lines)]
pub async fn purchase(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::super::requires_roles(
        command.user.id,
        &[GARAGE],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("garage purchase", options)),
        &mut interaction,
    )
    .await?;

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
            let Ok(Ok((Response::PurchaseShopAsset(Ok(plate)), _))) = events_request_2!(
                bootstrap::NC::get().await,
                synixe_events::garage::db,
                PurchaseShopAsset {
                    order: ShopOrder::Vehicle {
                        id,
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
                plate.expect("vehicle purchase must have plate"),
                command.user.id,
            ));
            interaction.reply("**Vehicle Purchased**\n\n").await
        }
        "addon" => {
            let Some(vehicle) = get_option!(options, "vehicle", String) else {
                return interaction
                    .reply("Required option not provided: vehicle")
                    .await;
            };
            let Ok(Ok((Response::PurchaseShopAsset(Ok(_)), _))) = events_request_2!(
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
                id, vehicle, command.user.id,
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
