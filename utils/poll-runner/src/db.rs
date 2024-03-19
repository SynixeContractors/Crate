use base64::{engine::general_purpose::STANDARD, Engine};
use bootstrap::DB;
use rsa::RsaPrivateKey;
use synixe_poll_runner::encrypt;
use uuid::Uuid;

pub async fn delete(id: Uuid) {
    println!("Deleting poll {id}");
    sqlx::query!(r#"DELETE FROM voting_polls WHERE id = $1"#, id)
        .execute(&*DB::get().await)
        .await
        .expect("should be able to delete poll");
}

pub async fn options(id: Uuid) -> Vec<(Uuid, String)> {
    sqlx::query!(
        r#"SELECT id, title FROM voting_options WHERE poll_id = $1"#,
        id
    )
    .fetch_all(&*DB::get().await)
    .await
    .expect("should be able to get options")
    .into_iter()
    .map(|option| (option.id, option.title))
    .collect::<Vec<_>>()
}

pub async fn votes(id: Uuid, private_key: &RsaPrivateKey) -> Vec<Uuid> {
    sqlx::query!(
        r#"SELECT encrypted_vote FROM voting_vote_box WHERE poll_id = $1"#,
        id
    )
    .fetch_all(&*DB::get().await)
    .await
    .expect("should be able to get votes")
    .into_iter()
    .map(|vote| {
        let vote = vote.encrypted_vote;
        let vote = STANDARD
            .decode(vote.as_bytes())
            .expect("should be able to decode");
        let vote = encrypt::decrypt(&vote, private_key);
        Uuid::from_slice(&vote).expect("should be able to decode vote")
    })
    .collect::<Vec<_>>()
}

pub async fn members(id: Uuid, private_key: &RsaPrivateKey) -> Vec<String> {
    sqlx::query!(
        r#"SELECT encrypted_ticket FROM voting_ticket_box WHERE poll_id = $1"#,
        id
    )
    .fetch_all(&*DB::get().await)
    .await
    .expect("should be able to get members")
    .into_iter()
    .map(|ticket| {
        let ticket = ticket.encrypted_ticket;
        let ticket = STANDARD
            .decode(ticket.as_bytes())
            .expect("should be able to decode");
        let ticket = encrypt::decrypt(&ticket, private_key);
        String::from_utf8(ticket).expect("should be able to decode ticket")
    })
    .collect::<Vec<_>>()
}
