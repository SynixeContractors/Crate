use std::fmt::Write;

use serenity::all::{
    CommandDataOptionValue, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, UserId,
};
use synixe_events::casino::db::Response;
use synixe_meta::discord::role::{JUNIOR, MEMBER};
use synixe_proc::events_request_5;

use rand::Rng;

use crate::{
    discord::{handler::brain::voice, interaction::Interaction, slash::ShouldAsk},
    get_option,
};

const MAX_BID: u64 = 500;

enum GameOption {
    Integer {
        name: &'static str,
        description: &'static str,
        min: u64,
        max: u64,
    },
    String {
        name: &'static str,
        description: &'static str,
        choices: &'static [(&'static str, &'static str)],
    },
}

struct Game {
    id: &'static str,
    description: &'static str,
    options: &'static [GameOption],
}

const GAMES: &[Game] = &[
    Game {
        id: "coinflip",
        description: "Flip a coin. Payout is 2x your bet.",
        options: &[],
    },
    Game {
        id: "diceroll",
        description: "Pick a number 1-6 on a die roll. Payout is 6x your bet.",
        options: &[GameOption::Integer {
            name: "number",
            description: "Your guess (1-6)",
            min: 1,
            max: 6,
        }],
    },
    Game {
        id: "carddraw",
        description: "Draw a card, pick red or black. Payout is 2x your bet.",
        options: &[GameOption::String {
            name: "color",
            description: "Pick red or black",
            choices: &[("red", "red"), ("black", "black")],
        }],
    },
    Game {
        id: "numberguess",
        description: "Guess a number between 1-10. Payout is 10x your bet.",
        options: &[GameOption::Integer {
            name: "number",
            description: "Your guess (1-10)",
            min: 1,
            max: 10,
        }],
    },
    Game {
        id: "lucky3",
        description: "Roll 3 dice, hope to get 3s. Payout for one 3 = 1.5x, two = 4x, three = 30x",
        options: &[],
    },
];

pub fn register() -> CreateCommand {
    let mut cmd = CreateCommand::new("casino").description("Gamble your money away");

    for game in GAMES {
        let mut sub_cmd =
            CreateCommandOption::new(CommandOptionType::SubCommand, game.id, game.description)
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Integer,
                        "bid",
                        "The amount of money to bet",
                    )
                    .min_int_value(1)
                    .max_int_value(MAX_BID)
                    .required(true),
                );

        for option in game.options {
            sub_cmd = match option {
                GameOption::Integer {
                    name,
                    description,
                    min,
                    max,
                } => sub_cmd.add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, *name, *description)
                        .min_int_value(*min)
                        .max_int_value(*max)
                        .required(true),
                ),
                GameOption::String {
                    name,
                    description,
                    choices,
                } => {
                    let mut opt =
                        CreateCommandOption::new(CommandOptionType::String, *name, *description)
                            .required(true);
                    for (choice_name, choice_value) in *choices {
                        opt = opt.add_string_choice(*choice_name, *choice_value);
                    }
                    sub_cmd.add_sub_option(opt)
                }
            };
        }

        cmd = cmd.add_option(sub_cmd);
    }

    cmd
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

    // Validate game exists
    if !GAMES.iter().any(|g| g.id == subcommand.name) {
        return Ok(());
    }

    let CommandDataOptionValue::SubCommand(options) = &subcommand.value else {
        return Ok(());
    };
    let game = subcommand.name.as_str();
    // The command definition bounds the bid to 1..=MAX_BID
    let Some(bid) = get_option!(options, "bid", Integer).and_then(|bid| i32::try_from(*bid).ok())
    else {
        return interaction.reply("Invalid bid").await;
    };
    let outcome = match game {
        "coinflip" => Some(coinflip(bid)),
        "diceroll" => get_option!(options, "number", Integer).map(|guess| diceroll(bid, *guess)),
        "carddraw" => get_option!(options, "color", String).map(|guess| carddraw(bid, guess)),
        "numberguess" => {
            get_option!(options, "number", Integer).map(|guess| numberguess(bid, *guess))
        }
        "lucky3" => Some(lucky3(bid)),
        _ => unreachable!("validated against GAMES above"),
    };
    let Some(outcome) = outcome else {
        return interaction.reply("Invalid options").await;
    };

    if !transact(command.user.id, -bid, game).await {
        return interaction
            .reply("Failed to place your bet, please try again later")
            .await;
    }
    if outcome.winnings > 0 && !transact(command.user.id, outcome.winnings, game).await {
        error!(
            "failed to pay out {} to {} for {game}",
            outcome.winnings, command.user.id
        );
        return interaction
            .reply("You won, but your winnings couldn't be paid out. Please contact staff.")
            .await;
    }
    let mut message = outcome.message;
    if let Some(situation) = reaction_situation(game, bid, outcome.winnings)
        && let Some(line) = voice::react(&situation).await
    {
        message = format!("{message}\n\n> {line}\n> -# Ctirad Brodsky, casino owner");
    }
    interaction.reply(message).await
}

/// Payouts at least this many times the bet get a reaction from Brodsky
const BIG_WIN_MULTIPLIER: i32 = 6;
/// The chance Brodsky gloats when someone loses a maximum bet
const MAX_LOSS_REACTION_PERCENT: u32 = 25;

/// What Brodsky reacts to after a game, if it's worth a reaction
fn reaction_situation(game: &str, bid: i32, winnings: i32) -> Option<String> {
    if winnings >= bid * BIG_WIN_MULTIPLIER {
        return Some(format!(
            "A contractor just won {} at your casino's {game} game, betting {}. You own the casino and have to pay it out.",
            bootstrap::format::money(winnings, false),
            bootstrap::format::money(bid, false),
        ));
    }
    let max_bid = i32::try_from(MAX_BID).unwrap_or(i32::MAX);
    if winnings == 0
        && bid == max_bid
        && rand::rng().random_range(0..100) < MAX_LOSS_REACTION_PERCENT
    {
        return Some(format!(
            "A contractor just lost the maximum bet of {} at your casino's {game} game. You own the casino.",
            bootstrap::format::money(bid, false),
        ));
    }
    None
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

/// Moves money for a game: a negative amount buys in, a positive one cashes out
async fn transact(member: UserId, amount: i32, game: &str) -> bool {
    let game = game.to_string();
    if amount < 0 {
        matches!(
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::casino::db,
                BuyIn {
                    member,
                    amount: -amount,
                    game,
                }
            )
            .await,
            Ok(Ok((Response::BuyIn(Ok(())), _)))
        )
    } else {
        matches!(
            events_request_5!(
                bootstrap::NC::get().await,
                synixe_events::casino::db,
                CashOut {
                    member,
                    amount,
                    game,
                }
            )
            .await,
            Ok(Ok((Response::CashOut(Ok(())), _)))
        )
    }
}

/// The result of a game: the total paid out (0 for a loss) and the message to show
struct Outcome {
    winnings: i32,
    message: String,
}

fn coinflip(bid: i32) -> Outcome {
    if rand::random() {
        let winnings = bid * 2;
        Outcome {
            winnings,
            message: format!("You won ${winnings}!"),
        }
    } else {
        Outcome {
            winnings: 0,
            message: format!("You lost ${bid}!"),
        }
    }
}

fn diceroll(bid: i32, guess: i64) -> Outcome {
    let roll = rand::random_range(1..=6);
    if roll == guess {
        let winnings = bid * 6;
        Outcome {
            winnings,
            message: format!("You rolled a {roll}! You guessed correctly! You won ${winnings}!"),
        }
    } else {
        Outcome {
            winnings: 0,
            message: format!("You rolled a {roll}! You guessed {guess}. You lost ${bid}!"),
        }
    }
}

fn carddraw(bid: i32, guess: &str) -> Outcome {
    let color = if rand::random() { "red" } else { "black" };
    if color == guess {
        let winnings = bid * 2;
        Outcome {
            winnings,
            message: format!("The card is {color}! You won ${winnings}!"),
        }
    } else {
        Outcome {
            winnings: 0,
            message: format!("The card is {color}! You lost ${bid}!"),
        }
    }
}

fn numberguess(bid: i32, guess: i64) -> Outcome {
    let number = rand::random_range(1..=10);
    if number == guess {
        let winnings = bid * 10;
        Outcome {
            winnings,
            message: format!(
                "The number was {number}! You guessed correctly! You won ${winnings}!"
            ),
        }
    } else {
        Outcome {
            winnings: 0,
            message: format!("The number was {number}! You guessed {guess}. You lost ${bid}!"),
        }
    }
}

fn lucky3(bid: i32) -> Outcome {
    let rolls: [u32; 3] = std::array::from_fn(|_| rand::random_range(1..=6));
    let mut message = format!(
        "You rolled: {} {} {}\n",
        number_to_emoji(rolls[0]),
        number_to_emoji(rolls[1]),
        number_to_emoji(rolls[2])
    );
    let threes = rolls.iter().filter(|&&r| r == 3).count();
    let winnings = match threes {
        0 => 0,
        1 => bid * 3 / 2,
        2 => bid * 4,
        _ => bid * 30,
    };
    let _ = match threes {
        0 => write!(message, "You got no threes! You lost ${bid}!"),
        1 => write!(message, "You got 1 three! You won ${winnings}!"),
        _ => write!(message, "You got {threes} threes! You won ${winnings}!"),
    };
    Outcome { winnings, message }
}
