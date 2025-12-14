use porter::{
    PassBuilder, PassType, PorterError,
    google::{
        CardRowTemplateInfo, CardRowTwoItems, CardTemplateOverride, ClassTemplateInfo,
        FieldReference, FieldSelector, GenericClass, GenericObject, GoogleWalletClient,
        GoogleWalletConfig, TemplateItem,
    },
};
use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    client::Context,
};
use synixe_events::gear::db::Response;
use synixe_meta::discord::{
    BRODSKY, GUILD,
    channel::LOG,
    role::{ACTIVE, STAFF},
};
use synixe_proc::events_request_2;

use crate::{
    discord::interaction::{Confirmation, Interaction},
    get_option, get_option_user,
};

use super::{AllowPublic, ShouldAsk};

pub fn register() -> CreateCommand {
    CreateCommand::new("bank")
        .description("Interact with the bank")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "balance", "View a member's balance")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to view the balance of (select Brodsky to view the company account)")
                .required(true)
            )
            .allow_public()
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "transfer", "Transfer money to another member")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to transfer money to")
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of money to transfer")
                    .min_int_value(1)
                    .max_int_value(10_000)
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "The reason for the transfer")
                    .required(true)
                )
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "fine", "Fine a member")
                .add_sub_option(CreateCommandOption::new(CommandOptionType::User, "member", "The member to fine")
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::Integer, "amount", "The amount of money to fine")
                    .min_int_value(1)
                    .max_int_value(10_000)
                    .required(true)
                )
                .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "reason", "The reason for the fine")
                    .required(true)
                )
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "wallet", "Create a Google Wallet for your bank balance")
        )
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let Some(subcommand) = command.data.options.first() else {
        warn!("No subcommand for bank provided");
        return Ok(());
    };
    if let CommandDataOptionValue::SubCommand(options) = &subcommand.value {
        match subcommand.name.as_str() {
            "balance" => balance(ctx, command, options).await?,
            "transfer" => transfer(ctx, command, options).await?,
            "fine" => fine(ctx, command, options).await?,
            "wallet" => wallet(ctx, command).await?,
            _ => unreachable!(),
        }
    }
    Ok(())
}

async fn balance(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Fetching balance...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };

    let Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch balance").await;
    };

    if user == &BRODSKY {
        return interaction
            .reply(format!(
                "<@{}> has:\n```Cash: {}\n```",
                BRODSKY,
                bootstrap::format::money(balance, false),
            ))
            .await;
    }

    let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch locker balance").await;
    };
    let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LoadoutBalance { member: *user }
    )
    .await
    else {
        return interaction.reply("Failed to fetch loudout balance").await;
    };
    interaction
        .reply(format!(
            "<@{}> has:\n```Cash:      {}\nLocker:    {}\nLoadout:   {}\nNet Worth: {}```",
            user,
            bootstrap::format::money(balance, false),
            bootstrap::format::money(locker_balance, false),
            bootstrap::format::money(loadout_balance, false),
            bootstrap::format::money(balance + locker_balance + loadout_balance, false)
        ))
        .await
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
async fn transfer(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    interaction.reply("Transferring money...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };

    super::requires_roles(
        command.user.id,
        &[ACTIVE],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Yes(("bank transfer", options)),
        &mut interaction,
    )
    .await?;

    if user != &synixe_meta::discord::BRODSKY && GUILD.member(&ctx, user).await?.user.bot {
        return interaction.reply("You can't transfer money to a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankTransferNew(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankTransferNew {
            source: command
                .member
                .as_ref()
                .expect("member should always exist on guild commands")
                .user
                .id,
            target: *user,
            #[allow(clippy::cast_possible_truncation)]
            amount: *amount as i32,
            reason: reason.clone(),
        }
    )
    .await
    else {
        return interaction.reply("Failed to transfer money").await;
    };
    let reply = format!(
        "Transferred {} to <@{}>",
        bootstrap::format::money(*amount as i32, false),
        user
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for transfer notification");
        return interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "<@{}> transferred you {}\n> {}",
                command
                    .member
                    .as_ref()
                    .expect("member should always exist on guild commands")
                    .user
                    .id,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send dm: {}", e);
        interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await?;
    }

    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!(
                "<@{}> transferred <@{}> {}\n> {}",
                command
                    .member
                    .as_ref()
                    .expect("member should always exist on guild commands")
                    .user
                    .id,
                user,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send log: {}", e);
    }

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cast_possible_truncation)]
async fn fine(
    ctx: &Context,
    command: &CommandInteraction,
    options: &[CommandDataOption],
) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), options);
    super::requires_roles(
        command.user.id,
        &[STAFF],
        &command
            .member
            .as_ref()
            .expect("member should always exist on guild commands")
            .roles,
        ShouldAsk::Deny,
        &mut interaction,
    )
    .await?;

    interaction.reply("Fining...").await?;
    let Some(user) = get_option_user!(options, "member") else {
        return interaction.reply("Invalid member").await;
    };
    if user != &synixe_meta::discord::BRODSKY && GUILD.member(&ctx, user).await?.user.bot {
        return interaction.reply("You can't fine a bot").await;
    }
    let Some(amount) = get_option!(options, "amount", Integer) else {
        return interaction.reply("Invalid amount").await;
    };
    let Some(reason) = get_option!(options, "reason", String) else {
        return interaction.reply("Invalid reason").await;
    };
    let Ok(Ok((Response::BankDepositNew(Ok(())), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankDepositNew {
            member: *user,
            #[allow(clippy::cast_possible_truncation)]
            amount: -*amount as i32,
            reason: format!("fine from {}: {reason}", command.user.id),
            id: None,
        }
    )
    .await
    else {
        return interaction.reply("Failed to fine").await;
    };
    let reply = format!(
        "Delivered a fine of {} to <@{}>.",
        bootstrap::format::money(*amount as i32, false),
        user,
    );
    interaction.reply(&reply).await?;

    let Ok(private_channel) = user.create_dm_channel(&ctx).await else {
        error!("Unable to create DM channel for fine notification");
        return interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await;
    };

    if let Err(e) = private_channel
        .say(
            &ctx.http,
            format!(
                "You were fined {}\n> {}",
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send dm: {}", e);
        interaction
            .reply(&format!("{reply}, but I wasn't able to notify them"))
            .await?;
    }

    if let Err(e) = LOG
        .say(
            &ctx.http,
            format!(
                "<@{}> fined <@{}> {}\n> {}",
                command.user.id,
                user,
                bootstrap::format::money(*amount as i32, false),
                reason.clone(),
            ),
        )
        .await
    {
        error!("failed to send log: {}", e);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn wallet(ctx: &Context, command: &CommandInteraction) -> serenity::Result<()> {
    let mut interaction = Interaction::new(ctx, command.clone(), &[]);
    if interaction.confirm("In order for this to work, you will have to give your Google account email to <@307524009854107648>. Have you already done this?").await? != Confirmation::Yes {
        return interaction
            .reply("Please contact <@307524009854107648> with your Google account email to set up your Google Wallet.")
            .await;
    }

    interaction
        .reply("Creating Google Wallet... This may take a moment.")
        .await?;

    let balance = match events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        BankBalance {
            member: command.user.id,
        }
    )
    .await
    {
        Ok(Ok((Response::BankBalance(Ok(Some(balance))), _))) => balance,
        Ok(Err(e)) => {
            error!("Failed to fetch balance: {}", e);
            return interaction
                .reply("Failed to fetch balance for Google Wallet")
                .await;
        }
        Err(e) => {
            error!("Failed to fetch balance: {}", e);
            return interaction
                .reply("Failed to fetch balance for Google Wallet")
                .await;
        }
        _ => {
            return interaction
                .reply("No balance found for Google Wallet")
                .await;
        }
    };
    let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LockerBalance {
            member: command.user.id
        }
    )
    .await
    else {
        return interaction.reply("Failed to fetch locker balance").await;
    };
    let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
        bootstrap::NC::get().await,
        synixe_events::gear::db,
        LoadoutBalance {
            member: command.user.id
        }
    )
    .await
    else {
        return interaction.reply("Failed to fetch loudout balance").await;
    };

    let config = GoogleWalletConfig {
        issuer_id: std::env::var("GOOGLE_WALLET_ISSUER_ID")
            .expect("GOOGLE_WALLET_ISSUER_ID must be set"),
        service_account_email: std::env::var("GOOGLE_WALLET_SERVICE_ACCOUNT")
            .expect("GOOGLE_WALLET_SERVICE_ACCOUNT must be set"),
        private_key: std::env::var("GOOGLE_WALLET_PRIVATE_KEY")
            .expect("GOOGLE_WALLET_PRIVATE_KEY must be set")
            .replace("\\n", "\n"),
    };

    let mut client = GoogleWalletClient::new(config.clone());

    let class_id = format!("{}.balance", config.issuer_id);
    let class = GenericClass {
        id: class_id.clone(),
        issuer_name: Some("Synixe Crate".to_string()),
        review_status: Some("UNDER_REVIEW".to_string()),
        class_template_info: Some(ClassTemplateInfo {
            card_template_override: Some(CardTemplateOverride {
                card_row_template_infos: Some(vec![
                    CardRowTemplateInfo {
                        one_item: None,
                        two_items: Some(CardRowTwoItems {
                            start_item: Some(TemplateItem {
                                first_value: Some(FieldSelector {
                                    fields: Some(vec![FieldReference {
                                        field_path: Some(
                                            "object.textModulesData['balance']".to_string(),
                                        ),
                                        date_format: None,
                                    }]),
                                }),
                                predefined_item: None,
                            }),
                            end_item: Some(TemplateItem {
                                first_value: Some(FieldSelector {
                                    fields: Some(vec![FieldReference {
                                        field_path: Some(
                                            "object.textModulesData['loadout']".to_string(),
                                        ),
                                        date_format: None,
                                    }]),
                                }),
                                predefined_item: None,
                            }),
                        }),
                        three_items: None,
                    },
                    CardRowTemplateInfo {
                        one_item: None,
                        two_items: Some(CardRowTwoItems {
                            start_item: Some(TemplateItem {
                                first_value: Some(FieldSelector {
                                    fields: Some(vec![FieldReference {
                                        field_path: Some(
                                            "object.textModulesData['locker']".to_string(),
                                        ),
                                        date_format: None,
                                    }]),
                                }),
                                predefined_item: None,
                            }),
                            end_item: Some(TemplateItem {
                                first_value: Some(FieldSelector {
                                    fields: Some(vec![FieldReference {
                                        field_path: Some(
                                            "object.textModulesData['net_worth']".to_string(),
                                        ),
                                        date_format: None,
                                    }]),
                                }),
                                predefined_item: None,
                            }),
                        }),
                        three_items: None,
                    },
                ]),
            }),
            details_template_override: None,
            list_template_override: None,
            card_barcode_section_details: None,
        }),
    };

    match client.create_generic_class(&class).await {
        Ok(_) => println!("✓ Class created"),
        Err(_) => match client.update_generic_class(&class_id, &class).await {
            Ok(_) => println!("✓ Class updated"),
            Err(e) => {
                error!("Failed to create or update class: {}", e);
                return interaction
                    .reply("Failed to create Google Wallet class")
                    .await;
            }
        },
    }

    let pass_id = format!("{}.balance_{}", config.issuer_id, command.user.id);
    let pass = PassBuilder::new(&pass_id, class_id.clone())
        .pass_type(PassType::Generic)
        .title("Synixe Account")
        .subtitle(
            command
                .user
                .nick_in(&ctx, GUILD)
                .await
                .unwrap_or_else(|| command.user.name.clone()),
        )
        .logo(
            "https://synixe.contractors/assets/img/logo-white.webp",
            Some("Synixe".to_string()),
        )
        .background_color("#ffd731")
        .field(
            "balance",
            "Cash Balance",
            bootstrap::format::money(balance, false),
        )
        .field(
            "locker",
            "Locker Balance",
            bootstrap::format::money(locker_balance, false),
        )
        .field(
            "loadout",
            "Loadout Balance",
            bootstrap::format::money(loadout_balance, false),
        )
        .field(
            "net_worth",
            "Net Worth",
            bootstrap::format::money(balance + locker_balance + loadout_balance, false),
        )
        .build();
    let google_pass: GenericObject = pass.clone().into();
    let created = match client.create_generic_object(&google_pass).await {
        Ok(c) => c,
        Err(PorterError::ApiError { status, message }) => {
            if status == 409 {
                match client.update_generic_object(&pass_id, &google_pass).await {
                    Ok(updated) => updated,
                    Err(e) => {
                        error!("Failed to update object: {}", e);
                        return interaction
                            .reply("Failed to create Google Wallet object")
                            .await;
                    }
                }
            } else {
                error!("Failed to create object: {}", message);
                return interaction
                    .reply("Failed to create Google Wallet object")
                    .await;
            }
        }
        Err(e) => {
            error!("Failed to create object: {}", e);
            return interaction
                .reply("Failed to create Google Wallet object")
                .await;
        }
    };

    let url = match client.generate_save_url(&created).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to generate save URL: {}", e);
            return interaction
                .reply("Failed to generate Google Wallet save URL")
                .await;
        }
    };

    interaction
        .reply(format!(
            "Your Google Wallet has been created! You can access it here: {url}\n\n"
        ))
        .await
}
