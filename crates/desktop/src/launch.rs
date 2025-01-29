#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch(
    client_info: &libalembic::settings::ClientInfo,
    account_info: &libalembic::settings::Account,
) -> anyhow::Result<()> {
    use libalembic::launch::Launcher;

    let mut launcher = Launcher::new(client_info, account_info);
    launcher.find_or_launch()?;
    launcher.inject()?;

    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch(
    client_info: &libalembic::settings::ClientInfo,
    account_info: &libalembic::settings::Account,
) -> anyhow::Result<()> {
    // TODO: Show some indication we can't launch on this platform
    Ok(())
}
