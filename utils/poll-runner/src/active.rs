use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use bootstrap::{DB, NC};
use dialoguer::Select;
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey},
    RsaPrivateKey,
};
use synixe_events::discord::{
    info,
    write::{self, DiscordContent, DiscordMessage},
};
use synixe_meta::discord::role::{ACTIVE, MEMBER, STAFF};
use synixe_poll_runner::encrypt;
use synixe_proc::events_request_5;
use uuid::Uuid;

#[allow(clippy::unused_async)]
#[allow(clippy::too_many_lines)]
pub async fn menu() {
    let db = DB::get().await;
    let staff = {
        let mut staff = HashMap::new();
        let Ok(Ok((info::Response::MembersByRole(Ok(members)), _))) = events_request_5!(
            NC::get().await,
            synixe_events::discord::info,
            MembersByRole { role: STAFF }
        )
        .await
        else {
            println!("Failed to get staff");
            return;
        };
        for member in members {
            staff.insert(member.user.id, member);
        }
        staff
    };
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
        .unwrap()
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
            .unwrap();
        if selection == active.len() {
            return;
        }
        let (id, title, description) = &active[selection];
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
        .unwrap()
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
        let option = Select::new()
            .with_prompt("Select Action")
            .items(if keys.len() >= 3 {
                &["Close Poll", "Manage Keys", "Delete"]
            } else {
                &["Close Poll (Unavailable)", "Manage Keys", "Delete"]
            })
            .interact()
            .unwrap();
        match option {
            0 => {
                if keys.len() < 3 {
                    println!("Not enough keys");
                    continue;
                }
                let private_key = encrypt::rebuild_key({
                    keys.into_iter()
                        .map(|(_, shard, private_key)| {
                            (
                                shard,
                                rsa::RsaPrivateKey::from_pkcs8_der(
                                    &STANDARD.decode(private_key.as_bytes()).unwrap(),
                                )
                                .unwrap(),
                            )
                        })
                        .collect()
                });
                println!("Collecting votes...");
                let votes = sqlx::query!(
                    r#"
                    SELECT encrypted_vote
                    FROM voting_vote_box
                    WHERE poll_id = $1
                    "#,
                    id
                )
                .fetch_all(&*db)
                .await
                .unwrap()
                .into_iter()
                .map(|vote| {
                    let vote = vote.encrypted_vote;
                    let vote = STANDARD.decode(vote.as_bytes()).unwrap();
                    let vote = encrypt::decrypt(&vote, &private_key);
                    Uuid::from_slice(&vote).unwrap()
                })
                .collect::<Vec<_>>();
                println!("Collecting members...");
                let members = sqlx::query!(
                    r#"
                    SELECT encrypted_ticket
                    FROM voting_ticket_box
                    WHERE poll_id = $1
                    "#,
                    id
                )
                .fetch_all(&*db)
                .await
                .unwrap()
                .into_iter()
                .map(|ticket| {
                    let ticket = ticket.encrypted_ticket;
                    let ticket = STANDARD.decode(ticket.as_bytes()).unwrap();
                    let ticket = encrypt::decrypt(&ticket, &private_key);
                    String::from_utf8(ticket).unwrap()
                })
                .collect::<Vec<_>>();
                println!("Collection options...");
                let options = sqlx::query!(
                    r#"
                    SELECT id, title
                    FROM voting_options
                    WHERE poll_id = $1
                    "#,
                    id
                )
                .fetch_all(&*db)
                .await
                .unwrap()
                .into_iter()
                .map(|option| (option.id, option.title))
                .collect::<Vec<_>>();
                let mut results = HashMap::new();
                for (id, _) in &options {
                    results.insert(id, 0);
                }
                for vote in votes {
                    *results.get_mut(&vote).unwrap() += 1;
                }
                let mut results = results.into_iter().collect::<Vec<_>>();
                results.sort_by(|(_, a), (_, b)| b.cmp(a));
                let mut message = format!("Results for {title}:\n");
                for (id, count) in &results {
                    let title = &options
                        .iter()
                        .find(|(option_id, _)| &option_id == id)
                        .unwrap()
                        .1;
                    message.push_str(&format!("{title}: {count}\n"));
                }
                message.push_str("\nWinning option: ");
                let (id, _) = results.first().unwrap();
                let title = &options
                    .iter()
                    .find(|(option_id, _)| &option_id == id)
                    .unwrap()
                    .1;
                message.push_str(title);
                message.push_str("\n\nParticipants:\n");
                for member in members {
                    let (member, _) = member.split_once(':').unwrap();
                    message.push_str(&format!("<@{member}>\n"));
                }
                println!("{message}");
                sqlx::query!(
                    r#"
                    DELETE FROM voting_polls
                    WHERE id = $1
                    "#,
                    id
                )
                .execute(&*db)
                .await
                .unwrap();
            }
            1 => {
                let option = Select::new().with_prompt("Select Action").items(&{
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
                });
                let selection = option.interact().unwrap();
                if selection == staff.len() {
                    continue;
                }
                let (staff_id, member) = staff.iter().nth(selection).unwrap();
                if keys
                    .iter()
                    .any(|(staff, _, _)| staff == &staff_id.to_string())
                {
                    if dialoguer::Confirm::new()
                        .with_prompt(format!("Remove key for {}?", member.display_name()))
                        .interact()
                        .unwrap()
                    {
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
                        .unwrap();
                    }
                } else {
                    let key: String = dialoguer::Input::new()
                        .with_prompt(format!("Enter private key for {}", member.display_name()))
                        .interact()
                        .unwrap();
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
                    .unwrap();
                }
            }
            2 => {
                if dialoguer::Confirm::new()
                    .with_prompt("Are you sure?")
                    .interact()
                    .unwrap()
                {
                    sqlx::query!(
                        r#"
                        DELETE FROM voting_polls
                        WHERE id = $1
                        "#,
                        id
                    )
                    .execute(&*db)
                    .await
                    .unwrap();
                }
            }
            _ => unreachable!(),
        }
    }
}
