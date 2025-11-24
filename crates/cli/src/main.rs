#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use libalembic::{launcher::{launcher::Launcher, windows::WindowsLauncher}, settings::AlembicSettings};

    let mut settings = AlembicSettings::new();
    let _ = settings.load();

    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let mut launcher = WindowsLauncher::new(
        settings.client,
        settings.servers[settings.accounts[0].server_index].clone(),
        settings.accounts[0].clone(),
        settings.dll.clone()
    );

    launcher.find_or_launch()?;
    launcher.inject()?;

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("ctrl+c received...");

    println!("Ejecting DLL and exiting...");
    launcher.eject()?;

    println!("Done. Exiting.");

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    use anyhow::bail;
    use libalembic::settings::SettingsManager;
    let _settings = SettingsManager::to_string()?;

    bail!("The CLI only works on Windows.");
}
