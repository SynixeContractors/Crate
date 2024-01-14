use async_openai::{config::OpenAIConfig, types::AssistantObject, Client};

mod functions;

const A_ID: &str = "asst_5axJz8sGfDkXKAgrweRNrDzy";

pub struct Chat {
    client: Client<OpenAIConfig>,
    assistant: Option<AssistantObject>,
}

impl Chat {
    pub async fn new() -> Self {
        let client = Client::new();
        let assistant = setup_assistant(&client).await;
        if let Some(assistant) = &assistant {
            info!("assistant: {}", assistant.id);
        } else {
            error!("failed to setup assistant");
        }
        Self { client, assistant }
    }
}

async fn setup_assistant(client: &Client<OpenAIConfig>) -> Option<AssistantObject> {
    match client.assistants().retrieve(A_ID).await {
        Ok(assistant) => Some(assistant),
        Err(e) => {
            error!("failed to retrieve assistant: {}", e);
            None
        }
    }
}
