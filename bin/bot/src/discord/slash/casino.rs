use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context,
    CreateCommand, CreateCommandOption,
};
use synixe_events::casino::db::Response;
use synixe_meta::discord::role::{JUNIOR, MEMBER};
use synixe_proc::events_request_5;

use crate::{
    discord::{interaction::Interaction, slash::ShouldAsk},
    get_option,
};

const MAX_BID: u64 = 500;

pub fn register() -> CreateCommand {
    CreateCommand::new("casino")
        .description("Gamble your money away")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "coinflip",
                "Flip a coin to double your money or lose it all",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bid",
                    "The amount of money to bet",
                )
                .min_int_value(1)
                .max_int_value(MAX_BID)
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "diceroll",
                "Pick a number 1-6 on a die roll",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bid",
                    "The amount of money to bet",
                )
                .min_int_value(1)
                .max_int_value(MAX_BID)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "number", "Your guess (1-6)")
                    .min_int_value(1)
                    .max_int_value(6)
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "carddraw",
                "Draw a card, pick red or black",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bid",
                    "The amount of money to bet",
                )
                .min_int_value(1)
                .max_int_value(MAX_BID)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::String, "color", "Pick red or black")
                    .required(true)
                    .add_string_choice("red", "red")
                    .add_string_choice("black", "black"),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "numberguess",
                "Guess a number between 1-10",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "bid",
                    "The amount of money to bet",
                )
                .min_int_value(1)
                .max_int_value(MAX_BID)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Integer, "number", "Your guess (1-10)")
                    .min_int_value(1)
                    .max_int_value(10)
                    .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "lucky3",
                "Roll 3 dice, if any of them is a 3 you win. One 3 = 1.5x, two 3 = 4x, three 3 = 30x",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "bid",
                        "The amount of money to bet",
                    )
                    .min_int_value(1)
                    .max_int_value(MAX_BID)
                    .required(true),
                ),
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for casino provided");
        return Ok(());
    };

    let mut interaction = Interaction::new(ctx, command.clone(), &[]).ephemeral(false);

    super::requires_roles(
        command.user.id,
        &[JUNIOR, MEMBER],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;

    if !can_play(command, &mut interaction).await {
        return Ok(());
    }

    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "coinflip" => coinflip(ctx, &mut interaction, command, options).await?,
            "diceroll" => diceroll(ctx, &mut interaction, command, options).await?,
            "carddraw" => carddraw(ctx, &mut interaction, command, options).await?,
            "numberguess" => numberguess(ctx, &mut interaction, command, options).await?,
            "lucky3" => lucky3(ctx, &mut interaction, command, options).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub async fn can_play(command: &CommandInteraction, interaction: &mut Interaction<'_>) -> bool {
    let Ok(Ok((synixe_events::gear::db::Response::BankBalance(Ok(Some(balance))), _))) =
        events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::gear::db,
            BankBalance {
                member: command.user.id,
            }
        )
        .await
    else {
        let _ = interaction
            .reply("Failed to get your balance, please try again later")
            .await;
        return false;
    };

    if balance < 1000 {
        let _ = interaction
            .reply("You need at least $1000 to play casino games")
            .await;
        false
    } else {
        true
    }
}

fn number_to_emoji(number: u32) -> &'static str {
    match number {
        1 => ":one:",
        2 => ":two:",
        3 => ":three:",
        4 => ":four:",
        5 => ":five:",
        6 => ":six:",
        7 => ":seven:",
        8 => ":eight:",
        9 => ":nine:",
        10 => ":keycap_ten:",
        _ => unreachable!(),
    }
}

async fn coinflip(
    _ctx: &Context,
    interaction: &mut Interaction<'_>,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(amount) = get_option!(options, "bid", Integer) else {
        return interaction.reply("Invalid bid").await;
    };
    // Do the buy in
    let Ok(Ok((Response::BuyIn(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        BuyIn {
            member: command.user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            game: "coinflip".to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    // Flip the coin
    let win = rand::random();
    #[allow(clippy::cast_possible_truncation)]
    if win {
        let amount = (*amount * 2) as i32;
        // Cash out the winnings
        let Ok(Ok((Response::CashOut(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::casino::db,
            CashOut {
                member: command.user.id,
                #[allow(clippy::cast_possible_truncation)]
                amount,
                game: "coinflip".to_string(),
            }
        )
        .await
        else {
            return interaction.reply("Failed to cash out winnings").await;
        };
        interaction.reply(format!("You won ${amount}!")).await?;
    } else {
        interaction.reply(format!("You lost ${amount}!")).await?;
    }

    Ok(())
}

async fn diceroll(
    _ctx: &Context,
    interaction: &mut Interaction<'_>,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(amount) = get_option!(options, "bid", Integer) else {
        return interaction.reply("Invalid bid").await;
    };
    let Some(guess) = get_option!(options, "number", Integer) else {
        return interaction.reply("Invalid number").await;
    };
    // Do the buy in
    let Ok(Ok((Response::BuyIn(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        BuyIn {
            member: command.user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            game: "diceroll".to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    // Roll the die (1-6)
    let roll = rand::random::<u32>() % 6 + 1;
    if roll == *guess as u32 {
        #[allow(clippy::cast_possible_truncation)]
        let amount = (*amount * 6) as i32;
        // Cash out the winnings
        let Ok(Ok((Response::CashOut(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::casino::db,
            CashOut {
                member: command.user.id,
                #[allow(clippy::cast_possible_truncation)]
                amount,
                game: "diceroll".to_string(),
            }
        )
        .await
        else {
            return interaction.reply("Failed to cash out winnings").await;
        };
        interaction
            .reply(format!(
                "You rolled a {roll}! You guessed correctly! You won ${amount}!"
            ))
            .await?;
    } else {
        interaction
            .reply(format!(
                "You rolled a {roll}! You guessed {guess}. You lost ${amount}!"
            ))
            .await?;
    }

    Ok(())
}

async fn carddraw(
    _ctx: &Context,
    interaction: &mut Interaction<'_>,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(amount) = get_option!(options, "bid", Integer) else {
        return interaction.reply("Invalid bid").await;
    };
    let Some(guess) = get_option!(options, "color", String) else {
        return interaction.reply("Invalid color").await;
    };
    // Do the buy in
    let Ok(Ok((Response::BuyIn(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        BuyIn {
            member: command.user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            game: "carddraw".to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    // Draw the card
    let card_is_red = rand::random();
    let actual_color = if card_is_red { "red" } else { "black" };
    if actual_color == guess.as_str() {
        #[allow(clippy::cast_possible_truncation)]
        let amount = (*amount * 2) as i32;
        // Cash out the winnings
        let Ok(Ok((Response::CashOut(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::casino::db,
            CashOut {
                member: command.user.id,
                #[allow(clippy::cast_possible_truncation)]
                amount,
                game: "carddraw".to_string(),
            }
        )
        .await
        else {
            return interaction.reply("Failed to cash out winnings").await;
        };
        interaction
            .reply(format!("The card is {actual_color}! You won ${amount}!"))
            .await?;
    } else {
        interaction
            .reply(format!("The card is {actual_color}! You lost ${amount}!"))
            .await?;
    }

    Ok(())
}

async fn numberguess(
    _ctx: &Context,
    interaction: &mut Interaction<'_>,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(amount) = get_option!(options, "bid", Integer) else {
        return interaction.reply("Invalid bid").await;
    };
    let Some(guess) = get_option!(options, "number", Integer) else {
        return interaction.reply("Invalid number").await;
    };
    // Do the buy in
    let Ok(Ok((Response::BuyIn(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        BuyIn {
            member: command.user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            game: "numberguess".to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    // Pick a random number 1-10
    let number = (rand::random::<u32>() % 10) + 1;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    if number == *guess as u32 {
        #[allow(clippy::cast_possible_truncation)]
        let amount = (*amount * 10) as i32;
        // Cash out the winnings
        let Ok(Ok((Response::CashOut(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::casino::db,
            CashOut {
                member: command.user.id,
                #[allow(clippy::cast_possible_truncation)]
                amount,
                game: "numberguess".to_string(),
            }
        )
        .await
        else {
            return interaction.reply("Failed to cash out winnings").await;
        };
        interaction
            .reply(format!(
                "The number was {number}! You guessed correctly! You won ${amount}!"
            ))
            .await?;
    } else {
        interaction
            .reply(format!(
                "The number was {number}! You guessed {guess}. You lost ${amount}!"
            ))
            .await?;
    }

    Ok(())
}

async fn lucky3(
    _ctx: &Context,
    interaction: &mut Interaction<'_>,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let Some(amount) = get_option!(options, "bid", Integer) else {
        return interaction.reply("Invalid bid").await;
    };
    // Do the buy in
    let Ok(Ok((Response::BuyIn(Ok(())), _))) = events_request_5!(
        bootstrap::NC::get().await,
        synixe_events::casino::db,
        BuyIn {
            member: command.user.id,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            game: "lucky3".to_string(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    // Roll 3 dice
    let rolls: Vec<u32> = (0..3).map(|_| (rand::random::<u32>() % 6) + 1).collect();
    let mut reply = format!(
        "You rolled: {} {} {}\n",
        number_to_emoji(rolls[0]),
        number_to_emoji(rolls[1]),
        number_to_emoji(rolls[2])
    );
    let threes = rolls.iter().filter(|&&r| r == 3).count();
    let winnings = match threes {
        0 => 0,
        1 => (*amount as f64 * 1.5) as i32,
        2 => (*amount as f64 * 4.0) as i32,
        3 => (*amount as f64 * 30.0) as i32,
        _ => unreachable!(),
    };
    if winnings > 0 {
        // Cash out the winnings
        let Ok(Ok((Response::CashOut(Ok(())), _))) = events_request_5!(
            bootstrap::NC::get().await,
            synixe_events::casino::db,
            CashOut {
                member: command.user.id,
                #[allow(clippy::cast_possible_truncation)]
                amount: winnings,
                game: "lucky3".to_string(),
            }
        )
        .await
        else {
            return interaction.reply("Failed to cash out winnings").await;
        };
        reply.push_str(&format!("You got {threes} threes! You won ${winnings}!"));
    } else {
        reply.push_str("You got no threes! You lost your bet!");
    }
    interaction.reply(reply).await
}
