// Cork is a Windows executable that can be built for either:
// - i686-pc-windows-msvc (32-bit MSVC, native Windows build)
// - x86_64-pc-windows-gnu (64-bit MinGW, cross-compile from Linux/macOS)
// - i686-pc-windows-gnu (32-bit MinGW, cross-compile from Linux/macOS)
#[cfg(not(target_os = "windows"))]
compile_error!("cork can only be built for Windows targets");

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
#[allow(unused_imports)]
use std::process::Command;

#[derive(Parser)]
#[command(name = "cork")]
#[command(about = "AC client launcher utility for Alembic", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a new AC client with optional DLL injection
    Launch {
        /// Path to acclient.exe
        #[arg(long)]
        client: String,

        /// Server hostname
        #[arg(long)]
        hostname: String,

        /// Server port
        #[arg(long)]
        port: String,

        /// Account username
        #[arg(long)]
        account: String,

        /// Account password
        #[arg(long)]
        password: String,

        /// Path to DLL to inject (e.g., C:\\Program Files (x86)\\Decal 3.0\\Inject.dll)
        #[arg(long)]
        dll: Option<String>,

        /// Optional function name to execute in the DLL after injection (e.g., "DecalStartup")
        #[arg(long)]
        function: Option<String>,
    },
}

#[cfg(target_os = "windows")]
fn launch_client_with_injection(
    client_path: &str,
    hostname: &str,
    port: &str,
    account: &str,
    password: &str,
    dll_path: Option<&str>,
    dll_function: Option<&str>,
) -> Result<()> {
    if dll_path.is_some() {
        println!("Cork: Launching AC client with DLL injection");
    } else {
        println!("Cork: Launching AC client (no DLL injection)");
    }
    println!("  Client: {}", client_path);
    println!("  Server: {}:{}", hostname, port);
    println!("  Account: {}", account);

    if let Some(dll) = dll_path {
        println!("  DLL: {}", dll);
        if let Some(func) = dll_function {
            println!("  Function: {}", func);
        }
    }

    // Verify that acclient.exe exists
    let client_file = Path::new(client_path);
    if !client_file.exists() {
        anyhow::bail!(
            "ERROR: AC client executable not found at path: {}\nPlease verify the --client path is correct.",
            client_path
        );
    }
    if !client_file.is_file() {
        anyhow::bail!(
            "ERROR: AC client path exists but is not a file: {}\nPlease provide a path to acclient.exe",
            client_path
        );
    }

    // Verify that the DLL exists (if provided)
    if let Some(dll) = dll_path {
        let dll_file = Path::new(dll);
        if !dll_file.exists() {
            anyhow::bail!(
                "ERROR: DLL not found at path: {}\nPlease verify the --dll path is correct.",
                dll
            );
        }
        if !dll_file.is_file() {
            anyhow::bail!(
                "ERROR: DLL path exists but is not a file: {}\nPlease provide a path to a valid DLL file.",
                dll
            );
        }
    }

    // Build the command line arguments
    let arguments = format!("-h {} -p {} -a {} -v {}", hostname, port, account, password);

    println!("\nStarting process...");

    if let Some(dll) = dll_path {
        libalembic::injector::launch_suspended_inject_and_resume(
            client_path,
            &arguments,
            dll,
            dll_function,
        )?;
        println!("Client launched and DLL injected successfully!");
    } else {
        libalembic::injector::launch_without_injection(client_path, &arguments)?;
        println!("Client launched successfully!");
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn launch_client_with_injection(
    _client_path: &str,
    _hostname: &str,
    _port: &str,
    _account: &str,
    _password: &str,
    _dll_path: Option<&str>,
    _dll_function: Option<&str>,
) -> Result<()> {
    anyhow::bail!("Cork client launching is only supported on Windows");
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Launch {
            client,
            hostname,
            port,
            account,
            password,
            dll,
            function,
        } => launch_client_with_injection(
            &client,
            &hostname,
            &port,
            &account,
            &password,
            dll.as_deref(),
            function.as_deref(),
        ),
    }
}
