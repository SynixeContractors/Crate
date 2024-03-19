use bootstrap::DB;
use dialoguer::Select;
use sqlx::query;

use crate::input;

#[allow(clippy::unused_async)]
pub async fn menu() {
    let db = DB::get().await;
    let name = input::text("Name");
    let description = input::text("Description");
    let mut options: Vec<String> = Vec::new();
    loop {
        let selection = Select::new()
            .with_prompt("Create Options")
            .items(&{
                let mut items = options
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<&str>>();
                items.push("Add Option");
                items.push("Done");
                items
            })
            .interact()
            .expect("should be able to select option");
        if selection == options.len() {
            let option = input::text("Option");
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
    if !input::confirm("Continue?") {
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
    .expect("should be able to create poll")
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
        .expect("should be able to create option");
    }
    println!("Poll Created");
}
