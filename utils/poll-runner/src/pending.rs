use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use bootstrap::{DB, NC};
use dialoguer::Select;
use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::DecodePublicKey};
use synixe_events::discord::{
    info,
    write::{self, DiscordContent, DiscordMessage},
};
use synixe_meta::discord::role::{ACTIVE, MEMBER, STAFF};
use synixe_poll_runner::encrypt;
use synixe_proc::events_request_5;

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
        let pending = sqlx::query!(
            r#"
            SELECT id, title, description
            FROM voting_polls
            WHERE public_key is null
            "#
        )
        .fetch_all(&*db)
        .await
        .unwrap()
        .into_iter()
        .map(|poll| (poll.id, poll.title, poll.description))
        .collect::<Vec<_>>();
        if pending.is_empty() {
            println!("No pending polls");
            return;
        }
        let selection = Select::new()
            .with_prompt("Select Poll")
            .items(&{
                let mut items = pending
                    .iter()
                    .map(|(_, title, description)| format!("{title} - {description}"))
                    .collect::<Vec<_>>();
                items.push("Done".to_string());
                items
            })
            .interact()
            .unwrap();
        if selection == pending.len() {
            return;
        }
        let (id, title, description) = &pending[selection];
        let keys = sqlx::query!(
            r#"
            SELECT staff, public_key
            FROM voting_keys
            WHERE poll_id = $1
            "#,
            id
        )
        .fetch_all(&*db)
        .await
        .unwrap()
        .into_iter()
        .map(|key| (key.staff, key.public_key))
        .collect::<Vec<_>>();
        let option = Select::new()
            .with_prompt("Select Action")
            .items(if keys.len() >= 5 {
                &["Open Poll", "Manage Keys", "Delete"]
            } else {
                &["Open Poll (Unavailable)", "Manage Keys", "Delete"]
            })
            .interact()
            .unwrap();
        match option {
            0 => {
                if keys.len() < 5 {
                    println!("Not enough keys");
                    continue;
                }
                let participants = {
                    let mut participants = HashMap::new();
                    let Ok(Ok((info::Response::MembersByRole(Ok(members)), _))) =
                        events_request_5!(
                            NC::get().await,
                            synixe_events::discord::info,
                            MembersByRole { role: STAFF }
                        )
                        .await
                    else {
                        println!("Failed to get staff");
                        return;
                    };
                    members
                        .into_iter()
                        // .filter(|member| member.roles.iter().any(|role| role == &BOT))
                        .filter(|member| member.user.id == 307_524_009_854_107_648)
                        .for_each(|member| {
                            participants.insert(member.user.id, member);
                        });
                    participants
                };
                if dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "Are you sure? The poll will be sent to {} members: {}",
                        participants.len(),
                        participants
                            .values()
                            .map(serenity::all::Member::display_name)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .interact()
                    .unwrap()
                {
                    let (private_key, encrypted_shards) = encrypt::generate_key({
                        let mut staff_keys = HashMap::new();
                        for (staff, key) in keys {
                            staff_keys.insert(
                                staff.parse().unwrap(),
                                rsa::RsaPublicKey::from_public_key_der(
                                    &STANDARD.decode(key.as_bytes()).unwrap(),
                                )
                                .unwrap(),
                            );
                        }
                        staff_keys
                    });
                    for (staff, shard) in encrypted_shards {
                        sqlx::query!(
                            r#"
                            UPDATE voting_keys SET shard = $1
                            WHERE poll_id = $2 AND staff = $3
                            "#,
                            shard,
                            id,
                            staff.to_string()
                        )
                        .execute(&*db)
                        .await
                        .unwrap();
                    }
                    sqlx::query!(
                        r#"
                        UPDATE voting_polls SET public_key = $1
                        WHERE id = $2
                        "#,
                        STANDARD.encode(private_key.to_public_key().to_pkcs1_der().unwrap()),
                        id
                    )
                    .execute(&*db)
                    .await
                    .unwrap();
                    for participant in participants.values() {
                        let ticket = encrypt::ticket(id, participant.user.id, &private_key);
                        let Ok(Ok((write::Response::UserMessage(Ok(())), _))) = events_request_5!(
                            NC::get().await,
                            synixe_events::discord::write,
                            UserMessage {
                                user: participant.user.id,
                                message: DiscordMessage {
                                    content: DiscordContent::Text(format!("## Poll: {title}\n{description}\n\nThis link is unique to you! Do not share it with anyone!\n[Vote Now!](https://baxter.synixe.contractors/vote/{ticket})"),),
                                    reactions: Vec::new() }
                            }
                        )
                        .await else {
                            println!("Failed to send poll to {}", participant.display_name());
                            continue;
                        };
                    }
                }
            }
            1 => {
                let option = Select::new().with_prompt("Select Action").items(&{
                    let mut items = staff
                        .iter()
                        .map(|(id, member)| {
                            if keys.iter().any(|(staff, _)| staff == &id.to_string()) {
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
                if keys.iter().any(|(staff, _)| staff == &staff_id.to_string()) {
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
                        .with_prompt(format!("Enter public key for {}", member.display_name()))
                        .interact()
                        .unwrap();
                    if key.is_empty() {
                        continue;
                    }
                    sqlx::query!(
                        r#"
                        INSERT INTO voting_keys (poll_id, staff, public_key)
                        VALUES ($1, $2, $3)
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
