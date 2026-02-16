fn main() {
    // Get git hash
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap().trim().to_string();

    // Get cargo version
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Set combined version for debug builds
    let debug_version = format!("{}-{}", cargo_version, git_hash);
    println!("cargo:rustc-env=DEBUG_VERSION={}", debug_version);
}
