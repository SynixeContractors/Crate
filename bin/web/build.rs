#[cfg(not(target_os = "windows"))]
const NPM_COMMAND: &str = "npm";
#[cfg(not(target_os = "windows"))]
const NPX_COMMAND: &str = "npx";

#[cfg(target_os = "windows")]
const NPM_COMMAND: &str = "npm.cmd";
#[cfg(target_os = "windows")]
const NPX_COMMAND: &str = "npx.cmd";

fn main() {
    let mut cmd = std::process::Command::new(NPM_COMMAND);
    cmd.arg("install");
    let output = cmd.output().expect("Failed to run npm");
    assert!(
        output.status.success(),
        "Failed to run npm: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut cmd = std::process::Command::new(NPX_COMMAND);
    cmd.arg("tailwindcss")
        .arg("-i")
        .arg("templates/index.css")
        .arg("-c")
        .arg("tailwind.config.js")
        .arg("-o")
        .arg("tailwind.css");
    let output = cmd.output().expect("Failed to run npx tailwind");
    assert!(
        output.status.success(),
        "Failed to run npx tailwind: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("cargo:rerun-if-changed=templates/");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=pacakge.json");
}
