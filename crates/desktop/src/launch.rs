#[cfg(all(target_os = "windows", target_env = "msvc"))]
pub fn try_launch(
    client_info: &libalembic::settings::ClientInfo,
    account_info: &libalembic::settings::Account,
) -> anyhow::Result<std::num::NonZero<u32>> {
    use libalembic::launch::Launcher;

    let mut launcher = Launcher::new(client_info, account_info);
    let pid = launcher.find_or_launch()?;
    launcher.inject()?;

    Ok(pid)
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
pub fn try_launch(
    client_info: &libalembic::settings::ClientInfo,
    account_info: &libalembic::settings::Account,
) -> anyhow::Result<std::num::NonZero<u32>> {
    // TODO: Show some indication we can't launch on this platform

    use std::num::NonZero;
    Ok(NonZero::new(0).unwrap())
}
