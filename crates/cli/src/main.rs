#[cfg(all(target_os = "windows", target_env = "msvc"))]
use libalembic::launch::Launcher;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn main() -> Result<(), anyhow::Error> {
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
fn main() -> Result<(), anyhow::Error> {
    use libalembic::settings::SettingsManager;

    let version = SettingsManager::get(|s| s.general.version.clone());
    let path = SettingsManager::get(|s| s.client.path.clone());
    println!("Settings version is {version}");
    println!("Settings path is {path}");
    println!("CLI only works on Windows.");

    Ok(())
}
