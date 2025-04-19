use std::{ffi::OsStr, path::PathBuf};

use async_openai::{
    Client,
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
};
use git2::Repository;
use rand::seq::IndexedRandom;
use serenity::all::{CreateAttachment, CreateMessage};
use synixe_meta::discord::channel::{LEADERSHIP, LOG, ONTOPIC};
use tokio::fs::File;

use crate::ArcCacheAndHttp;

#[allow(clippy::too_many_lines)]
pub async fn execute(client: ArcCacheAndHttp) {
    let Ok(files) = pull() else {
        if let Err(e) = LOG
            .say(
                client.as_ref(),
                "Failed to pull the tips repo for weekly tips",
            )
            .await
        {
            error!("Failed to send message to log: {}", e);
        }
        return;
    };

    'message: for (channel, name) in [(ONTOPIC, "on_topic"), (LEADERSHIP, "leadership")] {
        let directory = files.join(name);
        let paths: Vec<_> = std::fs::read_dir(directory)
            .expect("couldn't read pullled repo")
            .filter(|p| {
                p.as_ref()
                    .map(|p| p.path().extension() == Some(OsStr::new("md")))
                    .unwrap_or(false)
            })
            .collect();
        let file = {
            paths
                .choose(&mut rand::rng())
                .expect("no files in folder")
                .as_ref()
                .expect("File no longer exists")
        };
        let mut message = std::fs::read_to_string(file.path()).expect("Failed to read file");

        // ai stuff
        if let Ok(request) = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4o-mini")
            .messages(vec![
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(format!(
                        "{}\n\nRewrite the tips that you receive in your own voice and style, you will be sending them out to the contractors as a weekly reminder",
                        include_str!("postweeklytips-prompt.txt")
                    ))
                    .build()
                    .expect("prompt is valid"),
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(&message)
                    .build()
                    .expect("prompt is valid"),
            ])
            .build()
        {
            match Client::new().chat().create(request).await {
                Ok(response) => {
                    if let Some(content) = response
                        .choices
                        .first()
                        .expect("has message")
                        .message
                        .clone()
                        .content
                    {
                        message = content;
                    }
                }
                Err(e) => {
                    warn!("error with openai: {}", e);
                }
            }
        } else {
            warn!("issues with chatgtp");
        }
        // end ai stuff

        let message = format!("## Tip of the Week\n\n{message}");
        for ext in ["webp", "png", "jpg"] {
            let image_file = file.path().with_extension(ext);
            if image_file.exists() {
                if let Err(e) = channel
                    .send_message(
                        client.as_ref(),
                        CreateMessage::new().content(message).add_file(
                            CreateAttachment::file(
                                &File::open(&image_file).await.expect("file did exist"),
                                image_file
                                    .file_name()
                                    .unwrap_or_else(|| OsStr::new("image.png"))
                                    .to_string_lossy(),
                            )
                            .await
                            .expect("able to read and upload file"),
                        ),
                    )
                    .await
                {
                    error!("failed to send tip with image: {}", e);
                }
                continue 'message;
            }
        }
        if let Err(e) = channel.say(client.as_ref(), message).await {
            error!("Failed to send message to {}: {}", channel, e);
        }
    }
}

#[allow(clippy::unwrap_used)]
fn pull() -> Result<PathBuf, String> {
    let result = std::panic::catch_unwind(|| {
        let tmp = std::env::temp_dir().join("synixe-crate-tips");
        let repo = Repository::open(&tmp).unwrap_or_else(|_| {
            git2::build::RepoBuilder::new()
                .branch("main")
                .clone("https://github.com/synixecontractors/tips", &tmp)
                .map_err(|e| format!("Failed to clone repository: {e}"))
                .unwrap()
        });
        repo.find_remote("origin")
            .and_then(|mut r| r.fetch(&["main"], None, None))
            .map_err(|e| format!("Failed to fetch remote: {e}"))
            .unwrap();
        let fetch_head = repo
            .find_reference("FETCH_HEAD")
            .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
            .unwrap();
        let commit = repo
            .reference_to_annotated_commit(&fetch_head)
            .map_err(|e| format!("Failed to find FETCH_HEAD: {e}"))
            .unwrap();
        let analysis = repo
            .merge_analysis(&[&commit])
            .map_err(|e| format!("Failed to analyze merge: {e}"))
            .unwrap();
        if !analysis.0.is_up_to_date() && analysis.0.is_fast_forward() {
            let mut reference = repo
                .find_reference("refs/heads/main")
                .map_err(|e| format!("Failed to find reference: {e}"))
                .unwrap();
            reference
                .set_target(commit.id(), "Fast-Forward")
                .map_err(|e| format!("Failed to set reference: {e}"))
                .unwrap();
            repo.set_head("refs/heads/main")
                .map_err(|e| format!("Failed to set HEAD: {e}"))
                .unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| format!("Failed to checkout HEAD: {e}"))
                .unwrap();
        }
        tmp
    });
    result.map_err(|_| String::from("failed to pull"))
}
