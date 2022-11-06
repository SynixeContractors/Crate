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

use crate::discord::interaction::Interaction;

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
    let mut interaction = Interaction::new(ctx, command);
    interaction.reply("Fetching balance...").await;
    let member = if let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
    {
        user
    } else {
        panic!("Invalid member");
    };
    let Ok(((Response::BankBalance(Ok(balance)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: member.id,
        }
    )
    .await else {
        interaction.reply("Failed to fetch balance").await;
        return;
    };
    interaction
        .reply(format!(
            "<@{}> has ${}",
            member.id,
            bootstrap::format::money(balance)
        ))
        .await;
}

async fn transfer(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    options: &[CommandDataOption],
) {
    let mut interaction = Interaction::new(ctx, command);
    interaction.reply("Transferring money...").await;
    let member = if let CommandDataOptionValue::User(user, _member) = options
        .iter()
        .find(|option| option.name == "member")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
    {
        user
    } else {
        panic!("Invalid member");
    };
    if member.bot && member.id.0 != 1_028_418_063_168_708_638 {
        interaction.reply("You can't transfer money to a bot").await;
        return;
    }
    let amount = if let CommandDataOptionValue::Integer(amount) = options
        .iter()
        .find(|option| option.name == "amount")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
    {
        *amount
    } else {
        panic!("Invalid amount");
    };
    let reason = if let CommandDataOptionValue::String(reason) = options
        .iter()
        .find(|option| option.name == "reason")
        .unwrap()
        .resolved
        .as_ref()
        .unwrap()
    {
        reason
    } else {
        panic!("Invalid reason");
    };
    let Ok(((Response::BankTransferNew(Ok(_)), _), _)) = events_request!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source: command.member.as_ref().unwrap().user.id,
            target: member.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: amount as i32,
            reason: reason.clone(),
        }
    )
    .await else {
        interaction.reply("Failed to transfer money").await;
        return;
    };
    interaction
        .reply(format!(
            "Transferred ${} to <@{}>",
            bootstrap::format::money(amount),
            member.id
        ))
        .await;
}
