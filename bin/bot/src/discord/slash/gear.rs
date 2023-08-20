use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("gear")
        .description("Manage your gear")
        .create_option(|option| {
            option
                .name("repaint")
                .description("Repaint a weapon")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|option| {
                    option
                        .name("weapon")
                        .description("Select the weapon you want to modify")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
                .create_sub_option(|option| {
                    option
                        .name("paint")
                        .description("Select the new paint job for the weapon")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
                })
        })
}
