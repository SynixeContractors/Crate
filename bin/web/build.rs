#[cfg(not(target_os = "windows"))]
const COMMAND: &str = "tailwindcss";

#[cfg(target_os = "windows")]
const COMMAND: &str = "tailwindcss.cmd";

fn main() {
    let mut cmd = std::process::Command::new(COMMAND);
    cmd.arg("-i")
        .arg("templates/index.css")
        .arg("-c")
        .arg("tailwind.config.js")
        .arg("-o")
        .arg("tailwind.css");
    let output = cmd.output().expect("Failed to run tailwindcss");
    if !output.status.success() {
        panic!(
            "Failed to run tailwindcss: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("cargo:rerun-if-changed=templates/");
}
