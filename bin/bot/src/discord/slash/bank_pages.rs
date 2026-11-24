//! Paging through a member's bank history, transfers and requests with Newer and Older buttons

use serenity::{
    all::{ButtonStyle, ComponentInteraction, UserId},
    builder::{
        CreateActionRow, CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    client::Context,
};
use synixe_events::gear::db::{BankRequest, BankRequestStatus, Response, Transaction, Transfer};
use synixe_meta::discord::{GUILD, role::STAFF};
use synixe_proc::events_request_5;

/// Entries on a page
pub const PAGE_SIZE: usize = 10;
/// The most characters of a transfer's or request's reason shown, so a page fits in one message
const REASON_LENGTH: usize = 60;
/// Prefix of every paging button, followed by `{kind}:{member}:{page}`
const PREFIX: &str = "bank_page:";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    History,
    Transfers,
    Requests,
}

impl Kind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::History => "history",
            Self::Transfers => "transfers",
            Self::Requests => "requests",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "history" => Some(Self::History),
            "transfers" => Some(Self::Transfers),
            "requests" => Some(Self::Requests),
            _ => None,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::History => "transactions",
            Self::Transfers => "transfers",
            Self::Requests => "requests",
        }
    }
}

/// One page of a list, ready to show
pub struct Page {
    /// The message text, including the list
    pub content: String,
    pub has_older: bool,
}

/// The rows of a page, with whether there's another page after it. Fetching one more than the page size shows
/// whether there's an older page without a second query.
fn window<T>(mut rows: Vec<T>) -> (Vec<T>, bool) {
    let more = rows.len() > PAGE_SIZE;
    rows.truncate(PAGE_SIZE);
    (rows, more)
}

fn offset(page: usize) -> i64 {
    i64::try_from(page * PAGE_SIZE).unwrap_or(i64::MAX)
}

fn limit() -> i64 {
    i64::try_from(PAGE_SIZE + 1).unwrap_or(i64::MAX)
}

/// Fetches and lays out one page. `None` if the lookup fails.
pub async fn render(kind: Kind, member: UserId, page: usize) -> Option<Page> {
    let (lines, has_older) = match kind {
        Kind::History => {
            let Ok(Ok((Response::BankHistory(Ok(mut history)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankHistory { member }
            )
            .await
            else {
                return None;
            };
            history.sort_by_key(|t| std::cmp::Reverse(t.created));
            let start = page * PAGE_SIZE;
            let rest: Vec<Transaction> = history.into_iter().skip(start).collect();
            let (rows, more) = window(rest);
            (rows.iter().map(history_line).collect::<Vec<_>>(), more)
        }
        Kind::Transfers => {
            let Ok(Ok((Response::BankTransfers(Ok(rows)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankTransfers {
                    member,
                    limit: limit(),
                    offset: offset(page),
                }
            )
            .await
            else {
                return None;
            };
            let (rows, more) = window(rows);
            (
                rows.iter()
                    .map(|t| transfer_line(t, member))
                    .collect::<Vec<_>>(),
                more,
            )
        }
        Kind::Requests => {
            let Ok(Ok((Response::BankRequests(Ok(rows)), _))) = events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::gear::db,
                BankRequests {
                    member,
                    limit: limit(),
                    offset: offset(page),
                }
            )
            .await
            else {
                return None;
            };
            let (rows, more) = window(rows);
            (
                rows.iter()
                    .map(|r| request_line(r, member))
                    .collect::<Vec<_>>(),
                more,
            )
        }
    };

    let heading = format!("Latest {} for <@{member}>, page {}", kind.title(), page + 1);
    let content = if lines.is_empty() {
        if page == 0 {
            format!("No {} found for <@{member}>", kind.title())
        } else {
            format!("{heading}\nThat's the end of the list")
        }
    } else {
        let body = lines.join("\n");
        match kind {
            // History is a table, so it's set in a code block to keep the columns lined up
            Kind::History => format!("{heading}:\n```{body}```"),
            Kind::Transfers | Kind::Requests => format!("{heading}:\n{body}"),
        }
    };
    Some(Page { content, has_older })
}

fn history_line(t: &Transaction) -> String {
    let items = t
        .items
        .iter()
        .map(|i| format!("{} x{}", i.name, i.quantity))
        .collect::<Vec<_>>()
        .join(", ");
    let detail = match (items.is_empty(), t.reason.as_str()) {
        (true, _) => t.reason.clone(),
        (false, "shop purchase") => items,
        (false, reason) => format!("{items} ({reason})"),
    };
    let detail: String = detail.chars().take(120).collect();
    format!(
        "{}  {:>10}  {:<12}  {detail}",
        t.created.date(),
        bootstrap::format::money(t.amount, false),
        t.kind.label(),
    )
}

fn transfer_line(t: &Transfer, member: UserId) -> String {
    let (sign, direction, other) = if t.source == member {
        ("-", "to", t.target)
    } else {
        ("+", "from", t.source)
    };
    let reason: String = t.reason.chars().take(REASON_LENGTH).collect();
    format!(
        "`{}` **{sign}{}** {direction} <@{other}> - {reason}",
        t.created.date(),
        bootstrap::format::money(t.amount, false),
    )
}

fn request_line(r: &BankRequest, member: UserId) -> String {
    let (direction, other) = if r.requester == member {
        ("requested from", r.payer)
    } else {
        ("asked by", r.requester)
    };
    let status = match r.status {
        BankRequestStatus::Open => "⏳ open",
        BankRequestStatus::Paying => "⏳ paying",
        BankRequestStatus::Paid => "✅ paid",
        BankRequestStatus::Declined => "❌ declined",
    };
    let reason: String = r.reason.chars().take(REASON_LENGTH).collect();
    format!(
        "`{}` **{}** {direction} <@{other}> · {status} - {reason}",
        r.created.date(),
        bootstrap::format::money(r.amount, false),
    )
}

fn button(kind: Kind, member: UserId, page: usize, label: &str) -> CreateButton {
    CreateButton::new(format!("{PREFIX}{}:{member}:{page}", kind.as_str()))
        .style(ButtonStyle::Secondary)
        .label(label)
}

/// The Newer and Older buttons for a page, none if there's only one page
pub fn controls(kind: Kind, member: UserId, page: usize, has_older: bool) -> Vec<CreateActionRow> {
    let mut buttons = Vec::new();
    if page > 0 {
        buttons.push(button(kind, member, page - 1, "◀ Newer"));
    }
    if has_older {
        buttons.push(button(kind, member, page + 1, "Older ▶"));
    }
    if buttons.is_empty() {
        Vec::new()
    } else {
        vec![CreateActionRow::Buttons(buttons)]
    }
}

/// Handles a Newer or Older button, editing the list in place
pub async fn page_button(ctx: &Context, component: &ComponentInteraction) -> serenity::Result<()> {
    let Some((kind, member, page)) =
        component
            .data
            .custom_id
            .strip_prefix(PREFIX)
            .and_then(|rest| {
                let mut parts = rest.split(':');
                let kind = Kind::parse(parts.next()?)?;
                let member = UserId::new(parts.next()?.parse().ok()?);
                let page = parts.next()?.parse().ok()?;
                Some((kind, member, page))
            })
    else {
        error!("invalid bank page button: {}", component.data.custom_id);
        return Ok(());
    };

    // Anyone may page a public list, but someone else's requests are only for staff and the people in them
    let viewer = component.user.id;
    let viewer_is_staff = GUILD
        .member(ctx, viewer)
        .await
        .is_ok_and(|m| m.roles.contains(&STAFF));
    if kind == Kind::Requests && member != viewer && !viewer_is_staff {
        return component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Only staff can see other members' requests")
                        .ephemeral(true),
                ),
            )
            .await;
    }

    let Some(page_content) = render(kind, member, page).await else {
        return component
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("Failed to fetch that page, try again")
                        .ephemeral(true),
                ),
            )
            .await;
    };
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new()
                    .content(page_content.content)
                    .components(controls(kind, member, page, page_content.has_older)),
            ),
        )
        .await
}

/// The first page of a list, as a slash command reply with its buttons
pub async fn first_page(kind: Kind, member: UserId) -> Option<(String, Vec<CreateActionRow>)> {
    let page = render(kind, member, 0).await?;
    let components = controls(kind, member, 0, page.has_older);
    Some((page.content, components))
}
