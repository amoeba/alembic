#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch() -> anyhow::Result<()> {
    use libalembic::launch::Launcher;

    let mut launcher = Launcher::new();
    launcher.find_or_launch()?;
    launcher.inject()?;

    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch() -> anyhow::Result<()> {
    Ok(())
}
