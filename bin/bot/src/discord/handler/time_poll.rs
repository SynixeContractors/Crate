use serenity::{
    model::prelude::{message_component::MessageComponentInteraction, ChannelId},
    prelude::Context,
};

use crate::discord::interaction::{Generic, Interaction};

pub async fn run(ctx: &Context, component: &MessageComponentInteraction) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, Generic::Message(component), &[]);
    let once_a_week = interaction
        .choice(
            "I would prefer Synixe only played once a week?",
            &vec![
                ("Yes".to_string(), "Yes"),
                ("No".to_string(), "No"),
                ("Either".to_string(), "Either"),
            ],
        )
        .await
        .expect("answer")
        .expect("answer exist");
    let day = interaction
        .choice(
            "What day would you prefer Synixe to play on?",
            &vec![
                ("Friday".to_string(), "Friday"),
                ("Saturday".to_string(), "Saturday"),
                ("Sunday".to_string(), "Sunday"),
            ],
        )
        .await
        .expect("answer")
        .expect("answer exist");
    let hour_earlier = interaction.choice("Synixe is considering moving our mission to an hour earlier, would you be able to regularly attend this time?", &vec![("Yes".to_string(), "Yes"), ("No".to_string(), "No"), ("Maybe".to_string(), "Maybe")]).await.expect("answer").expect("answer exist");
    let radical = interaction.choice("Synixe is considering moving our missions to Saturday at <t:1683921600:t>, would you be able to regularly attend this time?", &vec![("Yes".to_string(), "Yes"), ("No".to_string(), "No"), ("Maybe".to_string(), "Maybe")]).await.expect("answer").expect("answer exist");
    ChannelId(1_106_732_128_797_990_952)
        .send_message(&ctx.http, |m| {
            m.content(format!(
                "Time Poll <@{}>\nOnce a week: {:?}\nDay: {:?}\nHour Earlier: {:?}\nRadical: {:?}",
                component.user.id, once_a_week, day, hour_earlier, radical
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
