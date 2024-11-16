use std::collections::HashMap;

use bootstrap::NC;
use serenity::all::{Member, UserId};
use synixe_events::discord::info;
use synixe_meta::discord::role::KEY_HOLDER;
use synixe_proc::events_request_5;

/// Get all staff members from the discord server
pub async fn get_keyholders() -> Option<HashMap<UserId, Member>> {
    let mut staff = HashMap::new();
    let Ok(Ok((info::Response::MembersByRole(Ok(members)), _))) = events_request_5!(
        NC::get().await,
        synixe_events::discord::info,
        MembersByRole { role: KEY_HOLDER }
    )
    .await
    else {
        println!("Failed to get staff");
        return None;
    };
    for member in members {
        staff.insert(member.user.id, member);
    }
    Some(staff)
}
