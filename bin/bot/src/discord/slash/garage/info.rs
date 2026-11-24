//! Garage vehicles' fuel and damage, and repairs

use std::fmt::Write;

use serenity::{
    all::{CommandDataOption, CommandInteraction, CreateAttachment},
    client::Context,
};
use synixe_events::garage::db::{Response, VehicleDetails};
use synixe_meta::discord::role::GARAGE;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{
    discord::{
        interaction::{Confirmation, Interaction},
        slash::ShouldAsk,
    },
    get_option,
};

/// A vehicle's condition, read from the state saved by `objectState_save`
#[derive(Debug, PartialEq)]
pub struct Condition {
    pub destroyed: bool,
    /// Fuel from 0 to 1. Only saved when it isn't full.
    pub fuel: f64,
    /// Damaged hitpoints with their damage from 0 to 1, worst first
    pub damage: Vec<(String, f64)>,
}

impl Condition {
    pub fn of(state: Option<&serde_json::Value>) -> Self {
        let state = state.cloned().unwrap_or_default();
        let mut damage = state["hits"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|hit| {
                let name = hit.get(0)?.as_str()?;
                // Saved as a number, or a string if it went through floatsToStr
                let value = hit.get(1).and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })?;
                (!name.is_empty() && value > 0.0).then(|| (name.to_string(), value))
            })
            .collect::<Vec<_>>();
        damage.sort_by(|a, b| b.1.total_cmp(&a.1));
        Self {
            destroyed: state["dead"].as_bool().unwrap_or(false),
            fuel: state["fuel"].as_f64().unwrap_or(1.0).clamp(0.0, 1.0),
            damage,
        }
    }

    /// One line on the vehicle's damage
    pub fn damage_summary(&self) -> String {
        if self.destroyed {
            return String::from("💀 Destroyed");
        }
        match self.damage.first() {
            None => String::from("No damage"),
            Some((worst, value)) => format!(
                "{} damaged part{}, worst {} {}",
                self.damage.len(),
                if self.damage.len() == 1 { "" } else { "s" },
                part_name(worst),
                percent(*value)
            ),
        }
    }

    /// The fuel level, in litres too when the vehicle's capacity is known
    pub fn fuel_summary(&self, capacity: i32) -> String {
        if capacity > 0 {
            format!(
                "{} ({:.0} l)",
                percent(self.fuel),
                self.fuel * f64::from(capacity)
            )
        } else {
            percent(self.fuel)
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn percent(fraction: f64) -> String {
    format!("{}%", (fraction * 100.0).round() as i64)
}

/// A hitpoint's name without Arma's `Hit` prefix, e.g. `HitEngine` as `Engine`
pub fn part_name(hitpoint: &str) -> &str {
    hitpoint
        .get(..3)
        .filter(|prefix| prefix.eq_ignore_ascii_case("hit"))
        .map_or(hitpoint, |_| &hitpoint[3..])
}

fn stored(vehicle: &VehicleDetails) -> &'static str {
    if vehicle.stored {
        "In garage"
    } else {
        "Out in the field"
    }
}

/// A list longer than this is sent as a file, keeping the message under Discord's 2000 characters
const LIST_MESSAGE_LENGTH: usize = 1800;

async fn fetch(plate: Option<String>) -> Option<Vec<VehicleDetails>> {
    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        FetchVehicleDetails { plate }
    )
    .await
    {
        Ok(Ok((Response::FetchVehicleDetails(Ok(vehicles)), _))) => Some(vehicles),
        _ => None,
    }
}

pub async fn list(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Checking the garage...").await?;
    let Some(vehicles) = fetch(None).await else {
        return interaction.reply("Error fetching vehicle assets").await;
    };
    if vehicles.is_empty() {
        return interaction.reply("No vehicle assets found").await;
    }
    let mut list = String::new();
    for vehicle in &vehicles {
        let condition = Condition::of(vehicle.state.as_ref());
        let _ = write!(
            list,
            "**{}** `{}` · {} · ⛽ {} · {}",
            vehicle.name,
            vehicle.plate,
            stored(vehicle),
            condition.fuel_summary(vehicle.fuel_capacity),
            condition.damage_summary(),
        );
        if vehicle.repair_cost > 0 && !condition.destroyed {
            let _ = write!(
                list,
                " · 🔧 ~{}",
                bootstrap::format::money(vehicle.repair_cost, false)
            );
        }
        list.push('\n');
    }
    let summary = format!("**Vehicle Assets** ({})", vehicles.len());
    if list.len() > LIST_MESSAGE_LENGTH {
        // Plain text in the file, without the bold markers
        interaction
            .reply_with_attachment(
                summary,
                CreateAttachment::bytes(list.replace("**", "").into_bytes(), "garage.txt"),
            )
            .await
    } else {
        interaction.reply(format!("{summary}\n{list}")).await
    }
}

pub async fn view(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    let Some(plate) = get_option!(options, "vehicle", String) else {
        return interaction.reply("Invalid vehicle").await;
    };
    let Some(vehicle) = fetch(Some(plate.clone()))
        .await
        .and_then(|vehicles| vehicles.into_iter().next())
    else {
        return interaction
            .reply("Vehicle not found, pick one from the list")
            .await;
    };
    let condition = Condition::of(vehicle.state.as_ref());

    let mut out = format!(
        "## {} `{}`\n{} · last updated <t:{}:R>\n",
        vehicle.name,
        vehicle.plate,
        stored(&vehicle),
        vehicle.updated.unix_timestamp()
    );
    if let Some(addon) = &vehicle.addon {
        let _ = writeln!(out, "**Addon** {addon}");
    }
    let _ = writeln!(
        out,
        "**Fuel** {}",
        condition.fuel_summary(vehicle.fuel_capacity)
    );
    let _ = writeln!(out, "**Damage** {}", condition.damage_summary());
    if !condition.destroyed && !condition.damage.is_empty() {
        let parts = condition
            .damage
            .iter()
            .map(|(name, value)| format!("{:<20} {:>4}", part_name(name), percent(*value)))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = writeln!(out, "```{parts}```");
    }

    if !condition.destroyed
        && let Ok(Ok((Response::RepairEstimate(Ok(groups)), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::garage::db,
            RepairEstimate {
                plate: vehicle.plate.clone()
            }
        )
        .await
    {
        let costs = groups
            .iter()
            .filter(|group| group.cost > 0)
            .map(|group| {
                format!(
                    "{} {}",
                    group.name,
                    bootstrap::format::money(group.cost, false)
                )
            })
            .collect::<Vec<_>>();
        if !costs.is_empty() {
            let _ = writeln!(out, "**Repairs** {} · `/garage repair`", costs.join(" · "));
        }
    }
    interaction.reply(out).await
}

pub async fn repair(
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
        ShouldAsk::Yes(("garage repair", options)),
        &mut interaction,
    )
    .await?;
    let Some(plate) = get_option!(options, "vehicle", String) else {
        return interaction.reply("Invalid vehicle").await;
    };
    let Some(group) = get_option!(options, "part", String).and_then(|id| Uuid::parse_str(id).ok())
    else {
        return interaction
            .reply("Pick the part to repair from the list")
            .await;
    };

    let Ok(Ok((Response::RepairEstimate(Ok(groups)), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        RepairEstimate {
            plate: plate.clone()
        }
    )
    .await
    else {
        return interaction.reply("Failed to price the repair").await;
    };
    let Some(estimate) = groups.into_iter().find(|g| g.id == group) else {
        return interaction
            .reply("Pick the part to repair from the list")
            .await;
    };
    if estimate.hitpoints.is_empty() {
        return interaction
            .reply(format!("Nothing in {} needs repairing", estimate.name))
            .await;
    }

    if interaction
        .confirm(&format!(
            "Repair {} on `{plate}` for {}? The company pays.",
            estimate.name,
            bootstrap::format::money(estimate.cost, false)
        ))
        .await?
        != Confirmation::Yes
    {
        return interaction.reply("Repair cancelled").await;
    }

    match events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::garage::db,
        Repair {
            member: command.user.id,
            plate: plate.clone(),
            group,
        }
    )
    .await
    {
        Ok(Ok((Response::Repair(Ok(cost)), _))) => {
            interaction
                .reply(format!(
                    "Repaired {} on `{plate}` for {}",
                    estimate.name,
                    bootstrap::format::money(cost, false)
                ))
                .await
        }
        // The database's reasons are written to be shown, e.g. the vehicle is out in the field
        Ok(Ok((Response::Repair(Err(reason)), _))) => interaction.reply(reason).await,
        _ => {
            interaction
                .reply("Failed to repair the vehicle, please try again later")
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn full_and_undamaged() {
        let condition = Condition::of(Some(&json!({})));
        assert_eq!(condition.fuel, 1.0);
        assert_eq!(condition.damage_summary(), "No damage");
        assert_eq!(condition.fuel_summary(200), "100% (200 l)");
    }

    #[test]
    fn damaged() {
        let state = json!({
            "fuel": 0.25,
            "hits": [["HitEngine", 0.8], ["HitLFWheel", "0.5"], ["", 1.0], ["HitHull", 0]],
        });
        let condition = Condition::of(Some(&state));
        assert_eq!(condition.fuel_summary(0), "25%");
        assert_eq!(condition.damage.len(), 2);
        assert_eq!(
            condition.damage_summary(),
            "2 damaged parts, worst Engine 80%"
        );
    }

    #[test]
    fn destroyed() {
        let condition = Condition::of(Some(&json!({ "dead": true })));
        assert_eq!(condition.damage_summary(), "💀 Destroyed");
    }

    #[test]
    fn part_names() {
        assert_eq!(part_name("HitEngine"), "Engine");
        assert_eq!(part_name("hitfuel"), "fuel");
        assert_eq!(part_name("Hi"), "Hi");
        assert_eq!(part_name("Engine"), "Engine");
    }
}
