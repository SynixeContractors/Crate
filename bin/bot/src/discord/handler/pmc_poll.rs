use serenity::{
    model::prelude::{message_component::MessageComponentInteraction, ChannelId},
    prelude::Context,
};

use crate::discord::interaction::{Generic, Interaction};

pub async fn run(ctx: &Context, component: &MessageComponentInteraction) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Message(component), &[]);
    let more_special = interaction
        .choice(
            "I would like to see more specials (Non-PMC) during the week.",
            &vec![
                ("Agree".to_string(), "Agree"),
                ("Disagree".to_string(), "Disagree"),
                ("Neutral".to_string(), "Neutral"),
            ],
        )
        .await
        .expect("answer")
        .expect("answer exist");
    let replace_subcon = interaction
        .choice(
            "I would prefer a special over a subcon if only one can be run in a week.",
            &vec![
                ("Agree".to_string(), "Agree"),
                ("Disagree".to_string(), "Disagree"),
                ("Neutral".to_string(), "Neutral"),
            ],
        )
        .await
        .expect("answer")
        .expect("answer exist");
    let weekend_special = interaction
        .choice(
            "I would like some sessions on weekend to be special (Non-PMC).",
            &vec![
                ("Agree".to_string(), "Agree"),
                ("Disagree".to_string(), "Disagree"),
                ("Neutral".to_string(), "Neutral"),
            ],
        )
        .await
        .expect("answer")
        .expect("answer exist");
    ChannelId(1_106_732_128_797_990_952)
        .send_message(&ctx.http, |m| {
            m.content(format!(
                "PMC Poll <@{}>\nMore Specials: {:?}\nReplace Subcon: {:?}\nWeekend Special: {:?}",
                component.user.id, more_special, replace_subcon, weekend_special
            ))
        })
        .await
        .expect("send to poll");
    interaction
        .reply("Thank you for your response!")
        .await
        .expect("reply");
    Ok(())
}
