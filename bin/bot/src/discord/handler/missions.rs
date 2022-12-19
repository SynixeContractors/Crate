use std::collections::HashMap;

use regex::Regex;
use serenity::{model::prelude::Message, prelude::Context};
use synixe_events::missions::db::Response;
use synixe_proc::events_request;
use time::Date;

pub async fn validate_aar(ctx: &Context, message: Message) {
    if !(message.content.starts_with("```") || message.content.ends_with("```")) {
        return;
    }
    let Ok(aar) = parse_aar(&message.content) else {
        if let Err(e) = message.reply(&ctx.http, ":confused: I couldn't parse that AAR. Please make sure you're using the template.").await {
            error!("Error replying to message: {}", e);
        };
        return;
    };
    if let Ok(Ok((Response::FindScheduledDate(Ok(scheduled)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::missions::db,
        FindScheduledDate {
            mission: aar.mission,
            date: aar.date,
        }
    )
    .await
    {
        let Some(scheduled) = scheduled else {
            if let Err(e) = message.reply(&ctx.http, ":confused: I couldn't find that mission on that date. Double check the date and mission name.").await {
                error!("Error replying to message: {}", e);
            };
            return;
        };
        if let Err(e) = message
            .reply(&ctx.http, ":white_check_mark: AAR validated!")
            .await
        {
            error!("Error replying to message: {}", e);
        };
        if let Ok(Ok((Response::SetScheduledAar(Ok(())), _))) = events_request!(
            bootstrap::NC::get().await,
            synixe_events::missions::db,
            SetScheduledAar {
                scheduled: scheduled.id,
                message_id: message.id,
            }
        )
        .await
        {
            info!("Set AAR for scheduled {}", scheduled.id);
        } else {
            error!("Failed to set AAR for scheduled {}", scheduled.id);
        };
    }
}

#[derive(Debug)]
struct Aar {
    mission: String,
    date: Date,
    contractors: Vec<String>,
    outcome: Outcome,
    payment: Payment,
}

#[derive(Debug)]
enum Outcome {
    Success,
    Partial,
    Failure,
}

#[derive(Debug, Default)]
struct Payment {
    no_combat: i32,
    light_combat: i32,
    medium_combat: i32,
    heavy_combat: i32,
}

fn parse_aar(content: &str) -> Result<Aar, String> {
    let lower = content.to_lowercase();
    let regex = Regex::new(r"(?m)(\d+)(?:.+?)(no|light|medium|heavy)").unwrap();
    let mut lines = HashMap::new();
    for line in lower.lines() {
        if line.is_empty() || !line.contains(": ") {
            continue;
        }
        let split = line.split_once(": ").unwrap();
        lines.insert(split.0, split.1);
    }
    let mission_type = {
        if lines.contains_key("contract") {
            "contract"
        } else if lines.contains_key("subcontract") {
            "subcontract"
        } else if lines.contains_key("training") {
            "training"
        } else if lines.contains_key("special") {
            "special"
        } else {
            return Err("Could not determine mission type. Valid types are Contract, Subcontract, Training, and Special.".to_string());
        }
    };
    let Some(mission_name) = lines.get(mission_type) else { return Err(format!("Could not find mission name for mission type {mission_type}")) };
    println!("Mission Name: {mission_name}");

    let Some(date) = lines.get("date") else { return Err("Could not find date.".to_string()) };

    let contractors = {
        let Some(contractors) = lines.get("contractors") else { return Err("Could not find contractors.".to_string()) };
        contractors
            .split(", ")
            .map(std::string::ToString::to_string)
            .collect()
    };

    let result = regex.captures_iter(&lower);
    let mut payment = Payment::default();
    for mat in result {
        let amount = mat.get(1).unwrap().as_str().parse::<i32>().unwrap();
        let kind = mat.get(2).unwrap().as_str();
        match kind {
            "no" => payment.no_combat = amount,
            "light" => payment.light_combat = amount,
            "medium" => payment.medium_combat = amount,
            "heavy" => payment.heavy_combat = amount,
            _ => return Err(format!("Unknown payment type: {kind}")),
        }
    }

    let outcome = {
        if lower.contains("operation successful")
            || lower.contains("operation completed")
            || lower.contains("operation success")
        {
            Outcome::Success
        } else if lower.contains("operation partial") {
            Outcome::Partial
        } else if lower.contains("operation failure") || lower.contains("operation failed") {
            Outcome::Failure
        } else {
            return Err("Could not determine mission outcome. Valid outcomes are Operation Successful, Operation Partial Success, and Operation Failure.".to_string());
        }
    };

    let Ok(date) = Date::parse(date, time::macros::format_description!("[year]-[month]-[day]")) else {
        return Err(format!("Could not parse date: {date}. Make sure it's in the format YYYY-MM-DD."));
    };

    Ok(Aar {
        mission: (*mission_name).to_string(),
        date,
        contractors,
        outcome,
        payment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aar() {
        let aar = parse_aar(
            r#"
```
Contract: Pusherman
Date: 2022-12-17
OL: Jake King
ELs: Carson Sering (KIA), Chaplain Yi (After Casualty), John Lamb (KIA)

Contractors: Jake King, Brett Harrison, Nathanial Greene, Carson Sering, Sean Miles, Chaplain Yi, Matias Jackson, John Brown, John Lamb
Assets Deployed: 2x Arcadian
Assets Lost: None
Casualties: John Lamb, Carson Sering

AAR: Contractors were tasked with destroying Cartel Assets on the island nation of Tanoa. Contractors began by assaulting the airfield. There was a handful of armed cartel members present at the airfield, which were taken out in the assault. A Cartel owned C-130, Mohawk Transport Helicopter, and Fuel Truck was destroyed. The ATC tower was resistant to demolition charges, so one was set in the tower itself to destroy the electronics inside of the tower itself. From the airfield, contractors regrouped at the FOB, and then began to make their way to the town of Tavu. A siege of the town began to neutralize the cartel presence within the town. In the early moments of contact with the cartel, both FTL's were KIA. After regrouping and triaging patients, the remaining cartel members were neutralized. 7 cartel owned RHIB's were destroyed, 2 cartel owned vans were destroyed, and 5 crates of cartel product were destroyed. Contractors successfully exfil'd back to the fob.

Operation Successful.

Payment request: 60 No Combat 30 Light Combat 45 Medium Combat 15 Heavy Combat
```
"#,
        );
        assert!(aar.is_ok());
        println!("{aar:?}");
    }
}
