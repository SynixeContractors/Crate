use bootstrap::DB;
use dialoguer::{Confirm, Input, Select};
use sqlx::query;

#[allow(clippy::unused_async)]
pub async fn menu() {
    let db = DB::get().await;
    let name: String = Input::new().with_prompt("Poll Name").interact().unwrap();
    let description: String = Input::new()
        .with_prompt("Poll Description")
        .interact()
        .unwrap();
    let mut options: Vec<String> = Vec::new();
    loop {
        let option = Select::new().with_prompt("Create Options").items(&{
            let mut items = options
                .iter()
                .map(std::string::String::as_str)
                .collect::<Vec<&str>>();
            items.push("Add Option");
            items.push("Done");
            items
        });
        let selection = option.interact().unwrap();
        if selection == options.len() {
            let option: String = Input::new().with_prompt("Option").interact().unwrap();
            options.push(option);
        } else if selection == options.len() + 1 {
            break;
        } else {
            options.remove(selection);
        }
    }
    println!("====");
    println!("Name: {name}");
    println!("Description: {description}");
    println!("Options: {options:?}");
    println!("====");
    if !Confirm::new().with_prompt("Continue?").interact().unwrap() {
        return;
    }
    let id = query!(
        r#"
        INSERT INTO voting_polls (title, description)
        VALUES ($1, $2)
        RETURNING id
        "#,
        name,
        description,
    )
    .fetch_one(&*db)
    .await
    .unwrap()
    .id;
    for option in options {
        query!(
            r#"
            INSERT INTO voting_options (poll_id, title)
            VALUES ($1, $2)
            "#,
            id,
            option,
        )
        .execute(&*db)
        .await
        .unwrap();
    }
    println!("Poll Created");
}
