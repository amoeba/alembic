#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn main() -> anyhow::Result<()> {
    use libalembic::{launch::Launcher, settings::SettingsManager};
    let _settings = SettingsManager::to_string()?;

    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // TODO: Pull config from somewhere
    // TODO: Make following code use that config

    let mut launcher = Launcher::new();
    launcher.find_or_launch()?;
    launcher.inject()?;

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("ctrl+c received...");

    launcher.eject()?;

    println!("Exiting.");

    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn main() -> anyhow::Result<()> {
    use anyhow::bail;
    use libalembic::settings::SettingsManager;
    let _settings = SettingsManager::to_string()?;

    bail!("The CLI only works on Windows.");
}
