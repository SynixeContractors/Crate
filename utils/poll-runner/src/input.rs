pub fn confirm(prompt: &str) -> bool {
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .interact()
        .expect("should be able to confirm")
}

pub fn select<'a>(options: &'a [&str], prompt: &str) -> &'a str {
    let index = dialoguer::Select::new()
        .with_prompt(prompt)
        .items(options)
        .interact()
        .expect("should be able to select");
    options[index]
}

pub fn text(prompt: &str) -> String {
    dialoguer::Input::new()
        .with_prompt(prompt)
        .interact()
        .expect("should be able to input")
}
