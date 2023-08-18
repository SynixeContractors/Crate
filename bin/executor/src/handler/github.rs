use async_trait::async_trait;
use octocrab::{params::orgs::Role, Octocrab};
use synixe_events::{
    github::executions::{Request, Response},
    respond,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::Invite { github } => {
                let Ok(token) = std::env::var("GITHUB_TOKEN") else {
                    respond!(msg, Response::Invite(Err("No GitHub token".to_string()))).await?;
                    return Err(anyhow::anyhow!("No GitHub token"));
                };
                let octo = Octocrab::builder().personal_token(token).build()?;
                let org = octo.orgs("SynixeContractors");
                if org.check_membership(github).await? {
                    Err(anyhow::anyhow!("User already in org"))
                } else {
                    org.add_or_update_membership(github, None)
                        .await?;
                    Ok(())
                }
            }
        }
    }
}
