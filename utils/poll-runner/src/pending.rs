use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD, Engine};
use bootstrap::{DB, NC};
use dialoguer::Select;
use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::DecodePublicKey};
use serenity::all::{Member, UserId};
use synixe_events::discord::{
    info,
    write::{self, DiscordContent, DiscordMessage},
};
use synixe_meta::discord::role::{ACTIVE, MEMBER};
use synixe_poll_runner::encrypt;
use synixe_proc::events_request_5;
use uuid::Uuid;

use crate::{db, discord, input};

#[allow(clippy::unused_async)]
#[allow(clippy::too_many_lines)]
pub async fn menu() {
    let db = DB::get().await;
    let staff = discord::get_staff()
        .await
        .expect("should be able to get staff over nats");
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
        .expect("should be able to fetch pending polls from db")
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
            .expect("should be able to select a poll");
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
        .expect("should be able to fetch keys from db")
        .into_iter()
        .map(|key| (key.staff, key.public_key))
        .collect::<Vec<_>>();
        let option = input::select(
            if keys.len() >= 4 {
                &["Open Poll", "Manage Keys", "Delete", "Done"]
            } else {
                &["Open Poll (Unavailable)", "Manage Keys", "Delete", "Done"]
            },
            "Pending Poll Menu",
        );
        match option {
            "Open Poll" | "Open Poll (Unavailable)" => {
                open_poll(keys, *id, title, description).await;
            }
            "Manage Keys" => manage_keys(staff.clone(), keys, *id).await,
            "Delete" => {
                if input::confirm("Are you sure?") {
                    db::delete(*id).await;
                }
            }
            "Done" => return,
            _ => unreachable!(),
        }
    }
}

async fn open_poll(keys: Vec<(String, String)>, id: Uuid, title: &str, description: &str) {
    let db = DB::get().await;
    if keys.len() < 4 {
        println!("Not enough keys");
        return;
    }
    let participants = {
        let mut participants = HashMap::new();
        let Ok(Ok((info::Response::MembersByRole(Ok(members)), _))) = events_request_5!(
            NC::get().await,
            synixe_events::discord::info,
            MembersByRole { role: MEMBER }
        )
        .await
        else {
            println!("Failed to get members");
            return;
        };
        members
            .into_iter()
            .filter(|member| member.roles.iter().any(|role| role == &ACTIVE))
            .for_each(|member| {
                participants.insert(member.user.id, member);
            });
        participants
    };
    if input::confirm(&format!(
        "Are you sure? The poll will be sent to {} members: {}",
        participants.len(),
        participants
            .values()
            .map(serenity::all::Member::display_name)
            .collect::<Vec<_>>()
            .join(", ")
    )) {
        let (private_key, encrypted_shards) = encrypt::generate_key({
            let mut staff_keys = HashMap::new();
            for (staff, key) in keys {
                staff_keys.insert(
                    staff.parse().expect("should be able to parse staff id"),
                    rsa::RsaPublicKey::from_public_key_der(
                        &STANDARD
                            .decode(key.as_bytes())
                            .expect("should be able to decode public key"),
                    )
                    .expect("should be able to decode public key"),
                );
            }
            staff_keys
        });
        let public_key = private_key.to_public_key();
        for (staff, shard) in encrypted_shards {
            sqlx::query!(
                r#"UPDATE voting_keys SET shard = $1 WHERE poll_id = $2 AND staff = $3"#,
                shard,
                id,
                staff.to_string()
            )
            .execute(&*db)
            .await
            .expect("should be able to update shard");
        }
        sqlx::query!(
            r#"UPDATE voting_polls SET public_key = $1 WHERE id = $2"#,
            STANDARD.encode(
                public_key
                    .to_pkcs1_der()
                    .expect("should be able to encode public key")
            ),
            id
        )
        .execute(&*db)
        .await
        .expect("should be able to update public key");
        for participant in participants.values() {
            let ticket = encrypt::ticket(&id, participant.user.id, &public_key);
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

async fn manage_keys(staff: HashMap<UserId, Member>, keys: Vec<(String, String)>, id: Uuid) {
    let db = DB::get().await;
    let selection = Select::new()
        .with_prompt("Select Action")
        .items(&{
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
        })
        .interact()
        .expect("should be able to select action");
    if selection == staff.len() {
        return;
    }
    let (staff_id, member) = staff.iter().nth(selection).expect("should have staff");
    if keys.iter().any(|(staff, _)| staff == &staff_id.to_string()) {
        if input::confirm(&format!("Remove key for {}?", member.display_name())) {
            sqlx::query!(
                r#"DELETE FROM voting_keys WHERE poll_id = $1 AND staff = $2"#,
                id,
                staff_id.to_string()
            )
            .execute(&*db)
            .await
            .expect("should be able to delete key");
        }
    } else {
        let key: String = input::text(&format!("Enter public key for {}", member.display_name()));
        if key.is_empty() {
            return;
        }
        sqlx::query!(
            r#"INSERT INTO voting_keys (poll_id, staff, public_key) VALUES ($1, $2, $3)"#,
            id,
            staff_id.to_string(),
            key
        )
        .execute(&*db)
        .await
        .expect("should be able to insert key");
    }
}
