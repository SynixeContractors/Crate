use serde_json::json;
use serenity::prelude::Context;

use crate::discord::utils::find_members;

use super::BrainFunction;

pub struct LookupId {}

#[async_trait::async_trait]
impl BrainFunction for LookupId {
    fn name(&self) -> &'static str {
        "names_lookup_members"
    }

    fn desc(&self) -> &'static str {
        "Looks up the ids of the given names, only members that exist in the company will be in the response"
    }

    fn args(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "names": {
                    "type": "array",
                    "description": "names to lookup ids",
                    "items": {
                        "type": "string",
                    },
                }
            }
        })
    }

    async fn run(&self, ctx: &Context, args: serde_json::Value) -> Option<serde_json::Value> {
        let names = args["names"]
            .as_array()?
            .iter()
            .map(|v| v.as_str().expect("valid name string").to_owned())
            .collect::<Vec<_>>();
        match find_members(ctx, &names).await {
            Ok(members) => Some(json!(members.0)),
            Err(e) => Some(json!({
                "error": e,
            })),
        }
    }
}
