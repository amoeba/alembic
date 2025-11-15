use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(name = "cork")]
#[command(about = "DLL injection utility for Alembic", long_about = None)]
struct Args {
    /// Process ID to inject into
    #[arg(long)]
    pid: u32,

    /// Path to the DLL to inject
    #[arg(long)]
    dll: String,
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn inject_dll(pid: u32, dll_path: &str) -> Result<()> {
    use dll_syringe::{process::OwnedProcess, Syringe};
    use std::path::Path;

    // Verify DLL exists
    let dll_path_obj = Path::new(dll_path);
    if !dll_path_obj.exists() {
        anyhow::bail!("DLL not found at path: {}", dll_path);
    }

    println!("Attempting to inject DLL into process {}", pid);
    println!("DLL path: {}", dll_path);

    // Get the target process
    let target_process = OwnedProcess::from_pid(pid)
        .with_context(|| format!("Failed to open process with PID {}", pid))?;

    // Create syringe and inject
    let syringe = Syringe::for_process(target_process);

    syringe
        .inject(dll_path)
        .with_context(|| format!("Failed to inject DLL: {}", dll_path))?;

    println!("Successfully injected DLL into process {}", pid);

    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn inject_dll(_pid: u32, _dll_path: &str) -> Result<()> {
    anyhow::bail!("DLL injection is only supported on Windows");
}

fn main() -> Result<()> {
    let args = Args::parse();

    inject_dll(args.pid, &args.dll)?;

    Ok(())
}
