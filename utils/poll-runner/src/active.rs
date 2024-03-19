use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use bootstrap::DB;
use dialoguer::Select;
use rsa::pkcs8::DecodePrivateKey;
use serde::{Deserialize, Serialize};
use synixe_poll_runner::encrypt;
use uuid::Uuid;

use crate::{db, discord, input};

#[derive(Debug, Serialize, Deserialize)]
pub struct Results {
    participants: Vec<String>,
    results: HashMap<String, u32>,
}

#[allow(clippy::unused_async)]
#[allow(clippy::too_many_lines)]
pub async fn menu() {
    let db = DB::get().await;
    let staff = discord::get_staff()
        .await
        .expect("should be able to get staff over nats");
    loop {
        let active = sqlx::query!(
            r#"
            SELECT id, title, description
            FROM voting_polls
            WHERE public_key is not null
            "#
        )
        .fetch_all(&*db)
        .await
        .expect("should be able to get active polls")
        .into_iter()
        .map(|poll| (poll.id, poll.title, poll.description))
        .collect::<Vec<_>>();
        if active.is_empty() {
            println!("No active polls");
            return;
        }
        let selection = Select::new()
            .with_prompt("Select Poll")
            .items(&{
                let mut items = active
                    .iter()
                    .map(|(_, title, description)| format!("{title} - {description}"))
                    .collect::<Vec<_>>();
                items.push("Done".to_string());
                items
            })
            .interact()
            .expect("should be able to select poll");
        if selection == active.len() {
            return;
        }
        let (id, title, _) = &active[selection];
        let keys = sqlx::query!(
            r#"
            SELECT staff, shard, private_key
            FROM voting_keys
            WHERE poll_id = $1
            "#,
            id
        )
        .fetch_all(&*db)
        .await
        .expect("should be able to get keys")
        .into_iter()
        .filter_map(|key| {
            if let Some(private_key) = key.private_key {
                if let Some(shard) = key.shard {
                    Some((key.staff, shard, private_key))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
        let option = input::select(
            if keys.len() >= 3 {
                &["Close Poll", "Manage Keys", "Delete", "Done"]
            } else {
                &["Close Poll (Unavailable)", "Manage Keys", "Delete", "Done"]
            },
            "Active Poll Menu",
        );
        match option {
            "Close Poll" | "Close Poll (Unavailable)" => {
                close_poll(keys, *id, title.clone()).await;
            }
            "Manage Keys" => {
                let selection = Select::new()
                    .with_prompt("Select Action")
                    .items(&{
                        let mut items = staff
                            .iter()
                            .map(|(id, member)| {
                                if keys.iter().any(|(staff, _, _)| staff == &id.to_string()) {
                                    format!("{} - Remove Key", member.display_name())
                                } else {
                                    format!("{} - Add Key", member.display_name())
                                }
                            })
                            .collect::<Vec<_>>();
                        items.push("Done".to_string());
                        items
                    })
                    .interact()
                    .expect("should be able to select action");
                if selection == staff.len() {
                    continue;
                }
                let (staff_id, member) = staff.iter().nth(selection).expect("should have staff");
                if keys
                    .iter()
                    .any(|(staff, _, _)| staff == &staff_id.to_string())
                {
                    if input::confirm(&format!("Remove key for {}?", member.display_name())) {
                        sqlx::query!(
                            r#"
                            DELETE FROM voting_keys
                            WHERE poll_id = $1 AND staff = $2
                            "#,
                            id,
                            staff_id.to_string()
                        )
                        .execute(&*db)
                        .await
                        .expect("should be able to delete key");
                    }
                } else {
                    let key =
                        input::text(&format!("Enter private key for {}", member.display_name()));
                    if key.is_empty() {
                        continue;
                    }
                    sqlx::query!(
                        r#"
                        UPDATE voting_keys SET private_key = $3
                        WHERE poll_id = $1 AND staff = $2
                        "#,
                        id,
                        staff_id.to_string(),
                        key
                    )
                    .execute(&*db)
                    .await
                    .expect("should be able to update key");
                }
            }
            "Delete" => {
                if input::confirm("Are you sure you want to delete this poll?") {
                    sqlx::query!(
                        r#"
                        DELETE FROM voting_polls
                        WHERE id = $1
                        "#,
                        id
                    )
                    .execute(&*db)
                    .await
                    .expect("should be able to delete poll");
                }
            }
            "Done" => return,
            _ => unreachable!(),
        }
    }
}

async fn close_poll(keys: Vec<(String, String, String)>, id: Uuid, title: String) {
    let db = DB::get().await;
    if keys.len() < 3 {
        println!("Not enough keys");
        return;
    }
    let private_key = encrypt::rebuild_key({
        keys.into_iter()
            .map(|(_, shard, private_key)| {
                (
                    shard,
                    rsa::RsaPrivateKey::from_pkcs8_der(
                        &STANDARD
                            .decode(private_key.as_bytes())
                            .expect("should be able to decode private key"),
                    )
                    .expect("should be able to decode private key"),
                )
            })
            .collect()
    });
    println!("Collecting votes...");
    let votes = db::votes(id, &private_key).await;
    println!("Collecting members...");
    let members = db::members(id, &private_key).await;
    println!("Collection options...");
    let options = db::options(id).await;
    let mut results = HashMap::new();
    for (id, _) in &options {
        results.insert(id, 0);
    }
    for vote in votes {
        *results.get_mut(&vote).expect("should be able to get vote") += 1;
    }
    let mut results = results.into_iter().collect::<Vec<_>>();
    results.sort_by(|(_, a), (_, b)| b.cmp(a));
    let mut message = format!("Results for {title}:\n");
    for (id, count) in &results {
        let title = &options
            .iter()
            .find(|(option_id, _)| &option_id == id)
            .expect("should be able to find option")
            .1;
        message.push_str(&format!("{title}: {count}\n"));
    }
    message.push_str("\nWinning option: ");
    let (winning_id, _) = results.first().expect("should have results");
    let winning_title = &options
        .iter()
        .find(|(option_id, _)| &option_id == winning_id)
        .expect("should be able to find option")
        .1;
    message.push_str(winning_title);
    message.push_str("\n\nParticipants:\n");
    for member in &members {
        let (member, _) = member.split_once(':').expect("should be able to split");
        message.push_str(&format!("<@{member}>\n"));
    }
    println!("{message}");
    sqlx::query!(
        r#"INSERT INTO voting_results (poll_id, title, data) VALUES ($1, $2, $3)"#,
        id,
        title,
        serde_json::to_value(&Results {
            participants: members
                .iter()
                .map(|member| {
                    let (member, _) = member.split_once(':').expect("should be able to split");
                    member.to_string()
                })
                .collect(),
            results: results
                .iter()
                .map(|(id, count)| {
                    (
                        options
                            .iter()
                            .find(|(option_id, _)| &option_id == id)
                            .expect("should be able to find option")
                            .1
                            .clone(),
                        *count,
                    )
                })
                .collect()
        })
        .expect("should be able to serialize results")
    )
    .execute(&*db)
    .await
    .expect("should be able to insert results");
    db::delete(id).await;
}
