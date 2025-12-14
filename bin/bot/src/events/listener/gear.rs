use porter::{
    PassBuilder, PassType,
    google::{GenericObject, GoogleWalletClient, GoogleWalletConfig},
};
use serenity::async_trait;
use synixe_events::gear::{db::Response, publish::Publish};
use synixe_meta::discord::GUILD;
use synixe_proc::events_request_2;

use crate::cache_http::CacheAndHttp;

use super::Listener;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::BalanceChanged {
                member,
                new_balance,
                reason: _,
            } => {
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
                let Ok(Ok((Response::LockerBalance(Ok(locker_balance)), _))) = events_request_2!(
                    bootstrap::NC::get().await,
                    synixe_events::gear::db,
                    LockerBalance { member: *member }
                )
                .await
                else {
                    return Ok(());
                };
                let Ok(Ok((Response::LoadoutBalance(Ok(loadout_balance)), _))) = events_request_2!(
                    bootstrap::NC::get().await,
                    synixe_events::gear::db,
                    LoadoutBalance { member: *member }
                )
                .await
                else {
                    return Ok(());
                };
                let class_id = format!("{}.balance", config.issuer_id);
                let pass_id = format!("{}.balance_{}", config.issuer_id, member);
                let pass = PassBuilder::new(&pass_id, class_id.clone())
                    .pass_type(PassType::Generic)
                    .title("Synixe Account")
                    .subtitle(
                        GUILD
                            .member(CacheAndHttp::get().as_ref(), *member)
                            .await
                            .map(|m| m.display_name().to_string())
                            .expect("Failed to get member display name"),
                    )
                    .logo(
                        "https://synixe.contractors/assets/img/logo-white.webp",
                        Some("Synixe".to_string()),
                    )
                    .background_color("#ffd731")
                    .field(
                        "balance",
                        "Cash Balance",
                        bootstrap::format::money(*new_balance, false),
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
                        bootstrap::format::money(
                            new_balance + locker_balance + loadout_balance,
                            false,
                        ),
                    )
                    .build();
                let google_pass: GenericObject = pass.clone().into();
                if let Err(e) = client.update_generic_object(&pass_id, &google_pass).await {
                    error!("Failed to update object: {}", e);
                }
                Ok(())
            }
        }
    }
}
