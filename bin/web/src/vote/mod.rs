use axum::{Router, extract::Path, response::Html, routing::get};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE},
};
use bootstrap::NC;
use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
use synixe_events::voting::db::Response;
use synixe_proc::events_request_5;
use tera::Context;
use uuid::Uuid;

use crate::template::Template;

pub fn router() -> Router {
    Router::new()
        .route(
            "/keys",
            get(|| async { Html(include_str!("keys.html").to_string()) }),
        )
        .route("/{ticket}", get(vote))
        .route("/cast/{ticket}/{id}", get(cast))
}

async fn vote(Path(ticket_raw): Path<String>) -> Html<String> {
    let ticket = TicketData::from_path(&ticket_raw);
    let Ok(Ok((Response::CheckTicket(Ok(voted)), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        CheckTicket {
            ticket: STANDARD.encode(ticket.data)
        }
    )
    .await
    else {
        println!("Failed to find poll");
        return Html("Error, poll not found".to_string());
    };
    if voted {
        return Html("Error, ticket already used".to_string());
    }
    let mut context: Context = Context::new();
    let Ok(Ok((Response::GetPoll(Ok(Some((_, title, description, _)))), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        GetPoll { poll: ticket.poll }
    )
    .await
    else {
        return Html("Error, poll not found".to_string());
    };
    let Ok(Ok((Response::GetOptions(Ok(options)), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        GetOptions { poll: ticket.poll }
    )
    .await
    else {
        println!("Failed to find poll");
        return Html("Error, poll not found".to_string());
    };
    context.insert("title", &title);
    context.insert("description", &description);
    context.insert("options", &options);
    context.insert("ticket", &ticket_raw);
    Html(
        Template::get()
            .render("vote/index.html", &context)
            .unwrap_or_else(|e| {
                error!("Error rendering template: {}", e);
                "Error".to_string()
            }),
    )
}

async fn cast(Path((ticket, option)): Path<(String, String)>) -> Html<String> {
    let ticket = TicketData::from_path(&ticket);
    let Ok(Ok((Response::CheckTicket(Ok(voted)), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        CheckTicket {
            ticket: STANDARD.encode(&ticket.data)
        }
    )
    .await
    else {
        println!("Failed to find poll");
        return Html("Error, poll not found".to_string());
    };
    if voted {
        return Html("Error, ticket already used".to_string());
    }
    let Ok(Ok((Response::GetPoll(Ok(Some((_, _, _, public_key)))), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        GetPoll { poll: ticket.poll }
    )
    .await
    else {
        return Html("Error, poll not found".to_string());
    };
    let Some(public_key) = public_key else {
        return Html("Error, poll not active".to_string());
    };
    let public_key = RsaPublicKey::from_pkcs1_der(
        &STANDARD
            .decode(public_key)
            .expect("should be able to decode public key"),
    )
    .expect("should be able to decode public key");
    let option = Uuid::parse_str(&option).expect("should be able to parse option");
    let Ok(Ok((Response::Vote(Ok(())), _))) = events_request_5!(
        NC::get().await,
        synixe_events::voting::db,
        Vote {
            poll: ticket.poll,
            ticket: STANDARD.encode(ticket.data),
            option: STANDARD.encode(synixe_poll_runner::encrypt::encrypt(
                option.as_bytes(),
                &public_key
            )),
        }
    )
    .await
    else {
        println!("Failed to vote");
        return Html("Error, failed to vote".to_string());
    };
    Html("Voted".to_string())
}

#[derive(Debug)]
pub struct TicketData {
    pub poll: Uuid,
    pub data: Vec<u8>,
}

impl TicketData {
    pub fn from_path(data: &str) -> Self {
        let data = URL_SAFE
            .decode(data.as_bytes())
            .expect("should be able to decode ticket");
        let poll_length = data[0] as usize;
        let poll = Uuid::from_slice(&data[1..=poll_length]).expect("should be able to parse uuid");
        let data = data[poll_length + 1..].to_vec();
        Self { poll, data }
    }
}
