use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::{
            ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
        },
        command::CommandOptionType,
    },
    prelude::Context,
};
use synixe_events::gear::db::Response;
use synixe_proc::events_request;

use crate::discord::interaction::{Generic, Interaction};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bank")
        .description("Interact with the bank")
        .create_option(|option| {
            option
                .name("balance")
                .description("View a member's balance")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to view the balance of (select Brodsky to view the company account)")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("transfer")
                .description("Transfer money to another member")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("member")
                        .description("The member to transfer money to")
                        .kind(CommandOptionType::User)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("amount")
                        .description("The amount of money to transfer")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(10_000)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("reason")
                        .description("The reason for the transfer")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "balance" => balance(ctx, command, &subcommand.options).await,
            "transfer" => transfer(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

async fn balance(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    interaction.reply("Fetching balance...").await;
    let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid member");
    };
    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: user.id,
        }
    )
    .await else {
        interaction.reply("Failed to fetch balance").await;
        return;
    };
    interaction
        .reply(format!(
            "<@{}> has ${}",
            user.id,
            bootstrap::format::money(balance)
        ))
        .await;
}

#[allow(clippy::cast_possible_truncation)]
async fn transfer(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, Generic::Application(command));
    interaction.reply("Transferring money...").await;
    let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid member");
    };
    if user.bot && user.id.0 != 1_028_418_063_168_708_638 {
        interaction.reply("You can't transfer money to a bot").await;
        return;
    }
    let CommandDataOptionValue::Integer(amount) = options
        .iter()
        .find(|option| option.name == "amount")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid amount");
    };
    let CommandDataOptionValue::String(reason) = options
        .iter()
        .find(|option| option.name == "reason")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap() else {
        panic!("Invalid reason");
    };
    let Ok(Ok((Response::BankTransferNew(Ok(_)), _))) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source: command.member.as_ref().unwrap().user.id,
            target: user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            reason: reason.clone(),
        }
    )
    .await else {
        interaction.reply("Failed to transfer money").await;
        return;
    };
    let reply = format!(
        "Transferred ${} to <@{}>",
        bootstrap::format::money(*amount as i32),
        user.id
    );
    interaction.reply(&reply).await;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for transfer notification");
        interaction.reply(&format!("{reply}, but I wasn't able to notify them")).await;
        return;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "<@{}> transferred you ${}\n> {}",
                command.member.as_ref().unwrap().user.id,
                bootstrap::format::money(*amount as i32),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send dm: {}", e);
        interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await;
    }
}
