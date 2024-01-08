use arma_rs::Group;
use serenity::model::prelude::UserId;
use synixe_events::reputation::db;
use synixe_proc::events_request_5;

use crate::{RUNTIME, SERVER_ID};

pub fn group() -> Group {
    Group::new()
        .command("friendly_shot", command_friendly_shot)
        .command("civilian_shot", command_civilian_shot)
        .command("unarmed_shot", command_unarmed_shot)
        .command("surrendering_shot", command_surrendering_shot)
        .command("captive_shot", command_captive_shot)
        .command("unconscious_shot", command_unconscious_shot)
        .command("building_damaged", command_building_damaged)
        .command("friendly_healed", command_friendly_healed)
        .command("unfriendly_healed", command_unfriendly_healed)
        .command("civilian_healed", command_civilian_healed)
}

fn command_friendly_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::FriendlyShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            FriendlyShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit friendly Shot over nats");
            return;
        };
    });
}

fn command_civilian_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::CivilianShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            CivilianShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit civilian Shot over nats");
            return;
        };
    });
}

fn command_unarmed_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::UnarmedShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            UnarmedShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit unarmed Shot over nats");
            return;
        };
    });
}

fn command_surrendering_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::SurrenderingShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            SurrenderingShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit surrendering Shot over nats");
            return;
        };
    });
}

fn command_captive_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::CaptiveShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            CaptiveShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit captive Shot over nats");
            return;
        };
    });
}

fn command_unconscious_shot(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::UnconsciousShot(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            UnconsciousShot {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit unconscious Shot over nats");
            return;
        };
    });
}

fn command_building_damaged(member: String, target: String, weapon: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::BuildingDamaged(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            BuildingDamaged {
                member: UserId::new(discord),
                target: target.to_string(),
                weapon: weapon.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit building damaged over nats");
            return;
        };
    });
}

fn command_friendly_healed(member: String, target: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::FriendlyHealed(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            FriendlyHealed {
                member: UserId::new(discord),
                target: target.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit friendly healed over nats");
            return;
        };
    });
}

fn command_unfriendly_healed(member: String, target: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::UnfriendlyHealed(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            FriendlyHealed {
                member: UserId::new(discord),
                target: target.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit friendly healed over nats");
            return;
        };
    });
}

fn command_civilian_healed(member: String, target: String) {
    if *SERVER_ID != "primary-main" {
        return;
    };
    let Ok(discord) = member.parse::<u64>() else {
        error!("failed to parse discord id");
        return;
    };
    RUNTIME.spawn(async move {
        let Ok(Ok((db::Response::CivilianHealed(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::reputation::db,
            CivilianHealed {
                member: UserId::new(discord),
                target: target.to_string(),
            }
        )
        .await
        else {
            error!("failed to submit civilian healed over nats");
            return;
        };
    });
}
