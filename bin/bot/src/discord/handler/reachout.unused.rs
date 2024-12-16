let ids = vec![
    201295133495263232,
    224587256629952512,
    249208434191368202,
    861450120389722123,
    166610300374614017,
    214483764179501056,
    330507656622243840,
    606854668198608929,
    231505608182857738,
    90681659204046848,
    346486596905861121,
    406978569227730945,
    738859765278703796,
    307524009854107648,
    182196433137565696,
    354031841600208896,
    800059544897585172,
    358146229626077187,
    1190791510086664334,
    333056722807554058,
    354676657262559232,
    303399327177637888,
    346696943885352960,
    158050236688891905,
    204052646598803456,
    148208964734156800,
    405550678090579970,
    566370729516597289,
    539627042292105228,
    217188164073291776,
    472738541512687618,
    151560187927330816,
    421124862313103361,
    91010861161787392,
    159477719288119296,
    474701046380232704,
    371762126479556618,
    201647692068159488,
    246028552519155722,
    95252491175727104,
    909886628472946719,
    502705617375592448,
    310919976339111937,
    254091079375126528,
    177423756581535744,
    466803213941866497,
    228031162004668416,
    301877267196411904,
    86996938385281024,
    516067271060488243,
    262400601017548801,
    718637777779949648,
    180815339804688384,
    429352430560477185,
    167082359697440768,
]
.into_iter()
.map(UserId::new)
.collect::<Vec<_>>();
// get all members not in the list
let nonplayers = GUILD
    .members_iter(&ctx.http)
    .boxed()
    .filter_map(|member| async {
        let member = member.expect("Cannot get member");
        if member.user.bot {
            return None;
        }
        // Joined more than a month ago
        if member.joined_at.is_some_and(|joined| {
            joined
                > Timestamp::now()
                    .checked_sub(time::Duration::days(30))
                    .expect("Cannot subtract")
                    .into()
        }) {
            return None;
        }
        if ids.contains(&member.user.id) {
            return None;
        }
        Some(member)
    })
    .collect::<Vec<_>>()
    .await;
LOG.say(
    &ctx.http,
    &format!(
        "Send a reach-out message to the following members: \n{}",
        nonplayers
            .iter()
            .map(|member| format!("<@{}>", member.user.id))
            .collect::<Vec<_>>()
            .join(" ")
    ),
)
.await
.expect("Cannot send message");
for member in nonplayers {
    member.user.direct_message(&ctx.http, CreateMessage::new()
        .content("Hey! It's been a long while since you've played Arma 3 with Synixe Contractors. We're reaching out to see if you're still wanting to be a member of our Discord server. We'd love to see you return in-game, but we understand if you've moved on. Please select one of the following options:\n\n🔫 I am currently busy or not interested in Arma, but I want to return in-game in the future!\n👀 I am not interested in playing, but I wish to remain in the Discord for now\n👋 I am no longer interested in being a part of Synixe Contractors and would like to leave the Discord server\n\nIf you have any questions or concerns, please feel free to reach out to a staff member. We hope to see you back in the future! You can always find us at <https://synixe.contractors>")
        .components(vec![CreateActionRow::Buttons(vec![
            CreateButton::new("reachout_yes")
                .style(ButtonStyle::Primary)
                .emoji(ReactionType::Unicode("🔫".to_string())),
            CreateButton::new("reachout_maybe")
                .style(ButtonStyle::Secondary)
                .emoji(ReactionType::Unicode("👀".to_string())),
            CreateButton::new("reachout_no")
                .style(ButtonStyle::Danger)
                .emoji(ReactionType::Unicode("👋".to_string())),
        ])])
        ).await.expect("Cannot send message");
}
