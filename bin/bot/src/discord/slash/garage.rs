use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType,
    },
    prelude::Context,
};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("garage")
        .description("Interact with the garage")
        .create_option(|option| {
            option
                .name("view")
                .description("View the garage inventory")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("purchase")
                .description("purchase an asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("asset")
                        .description("The asset to purchase")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
        .create_option(|option| {
            option
                .name("modify")
                .description("Modify an asset")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("attach")
                        .description("Attach an asset to a vehicle")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("dettach")
                        .description("Dettach an asset from a vehicle")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("vehicle")
                        .description("The vehicle in question")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        })
}

pub async fn run(ctx: &Context, command: &ApplicationCommandInteraction) {
    let subcommand = command.data.options.first().unwrap();
    if subcommand.kind == CommandOptionType::SubCommand {
        match subcommand.name.as_str() {
            "view" => view(ctx, command, &subcommand.options).await,
            // "purchase" => purchase(ctx, command, &subcommand.options).await,
            // "modify" => modify(ctx, command, &subcommand.options).await,
            _ => unreachable!(),
        }
    }
}

async fn view(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    _options: &[ApplicationCommandInteractionDataOption],
) {
    let _ = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content("This is the garage view command")
                })
        })
        .await;
}
