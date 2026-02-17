use anyhow::{bail, Context};
use libalembic::settings::SettingsManager;

pub fn inject() -> anyhow::Result<()> {
    use libalembic::settings::ClientConfigType;
    use std::process::Command;

    println!("Running cork to find and inject into acclient.exe...");
    println!();

    // Get selected client config and index
    let (client_config, client_idx) =
        SettingsManager::get(|s| (s.get_selected_client().cloned(), s.selected_client));

    let client_config = match client_config {
        Some(config) => config,
        None => {
            bail!("No client selected. Use 'alembic client select <index>' to select a client.");
        }
    };

    // Only Wine clients are supported for now
    let wine_config = match client_config {
        ClientConfigType::Wine(config) => config,
        ClientConfigType::Windows(_) => {
            bail!("Inject command currently only supports Wine clients");
        }
    };

    // Get selected DLL config for the selected client
    let dll_config = match client_idx {
        Some(idx) => SettingsManager::get(|s| s.get_client_selected_dll(idx).cloned()),
        None => None,
    };

    let dll_path = match dll_config {
        Some(config) => config.dll_path.display().to_string(),
        None => {
            bail!(
                "No DLL selected. Use 'alembic client dll --client <index> select <dll_index>' to select a DLL for injection."
            );
        }
    };

    // Get cork.exe path
    let cork_path = std::env::current_exe()
        .context("Failed to get current executable path")?
        .parent()
        .context("Failed to get executable directory")?
        .join("cork.exe");

    if !cork_path.exists() {
        bail!("cork.exe not found at {:?}. Make sure it's built with: cargo build --package cork --target i686-pc-windows-gnu", cork_path);
    }

    println!("Client: {}", wine_config.name);
    if let Some(prefix) = wine_config.launch_command.env.get("WINEPREFIX") {
        println!("Wine prefix: {}", prefix);
    }
    println!("DLL: {}", dll_path);
    println!("Cork path: {}", cork_path.display());
    println!();

    // Run cork.exe under wine using the launch_command configuration
    let launch_cmd = &wine_config.launch_command;
    let mut cmd = Command::new(&launch_cmd.program);

    // Add pre-args (e.g., for flatpak: "run", "--command=wine", "net.lutris.Lutris")
    for arg in &launch_cmd.args {
        cmd.arg(arg);
    }

    // Set environment variables
    for (key, value) in &launch_cmd.env {
        cmd.env(key, value);
    }

    cmd.arg(cork_path.to_str().context("Invalid cork path")?);
    cmd.arg("--dll").arg(&dll_path);

    let output = cmd
        .output()
        .context("Failed to execute cork.exe under wine")?;

    println!("Cork output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.stderr.is_empty() {
        println!("Cork stderr:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
        println!("Cork completed successfully");
        Ok(())
    } else {
        bail!("Cork exited with status: {}", output.status);
    }
}
