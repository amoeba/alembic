// Enforce that cork can ONLY be built for 32-bit Windows (i686-pc-windows-msvc)
#[cfg(not(all(target_arch = "x86", target_os = "windows", target_env = "msvc")))]
compile_error!("cork can only be built for i686-pc-windows-msvc target. Use: cargo build --target i686-pc-windows-msvc -p cork");

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use dll_syringe::process::{OwnedProcess, Process};

#[derive(Parser)]
#[command(name = "cork")]
#[command(about = "AC client launcher utility for Alembic", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch a new AC client with DLL injection
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
        dll: String,

        /// Optional function name to execute in the DLL after injection (e.g., "DecalStartup")
        #[arg(long)]
        function: Option<String>,
    },
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn launch_client_with_injection(
    client_path: &str,
    hostname: &str,
    port: &str,
    account: &str,
    password: &str,
    dll_path: &str,
    dll_function: Option<&str>,
) -> Result<()> {
    println!("Cork: Launching AC client with DLL injection");
    println!("  Client: {}", client_path);
    println!("  Server: {}:{}", hostname, port);
    println!("  Account: {}", account);
    println!("  DLL: {}", dll_path);
    if let Some(func) = dll_function {
        println!("  Function: {}", func);
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
    println!("✓ AC client executable found");

    // Verify that the DLL exists
    let dll_file = Path::new(dll_path);
    if !dll_file.exists() {
        anyhow::bail!(
            "ERROR: DLL not found at path: {}\nPlease verify the --dll path is correct.",
            dll_path
        );
    }
    if !dll_file.is_file() {
        anyhow::bail!(
            "ERROR: DLL path exists but is not a file: {}\nPlease provide a path to a valid DLL file.",
            dll_path
        );
    }
    println!("✓ DLL file found");

    // Build the command line arguments
    let arguments = format!("-h {} -p {} -a {} -v {}", hostname, port, account, password);

    println!("\nStarting process...");
    libalembic::injector::launch_suspended_inject_and_resume(
        client_path,
        &arguments,
        dll_path,
        dll_function,
    )?;

    println!("Client launched and DLL injected successfully!");
    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn launch_client_with_injection(
    _client_path: &str,
    _hostname: &str,
    _port: &str,
    _account: &str,
    _password: &str,
    _dll_path: &str,
    _dll_function: Option<&str>,
) -> Result<()> {
    anyhow::bail!("Cork client launching is only supported on Windows");
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn find_acclient_windows() -> Result<()> {
    println!("Cork: Searching for acclient.exe process using Windows API");

    if let Some(process) = OwnedProcess::find_first_by_name("acclient") {
        match process.pid() {
            Ok(pid) => {
                println!("Found acclient.exe!");
                println!("  Process ID: {}", pid);
                println!("\nDone. Exiting without injection.");
                return Ok(());
            }
            Err(e) => {
                println!("Found acclient.exe but couldn't get PID: {}", e);
            }
        }
    } else {
        println!("No acclient.exe process found");
    }

    println!("\nDone. Exiting without injection.");
    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn find_acclient_windows() -> Result<()> {
    println!("Cork: Searching for acclient.exe using tasklist");

    // Call tasklist.exe to get the process list
    let output = Command::new("tasklist.exe").output()?;

    if !output.status.success() {
        anyhow::bail!("tasklist.exe failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("tasklist output:\n{}", stdout);

    // Parse output looking for acclient.exe
    // Format is: acclient.exe,32
    for line in stdout.lines() {
        if line.starts_with("acclient.exe,") {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                if let Ok(pid) = parts[1].trim().parse::<u32>() {
                    println!("\nFound acclient.exe!");
                    println!("  Process ID: {}", pid);
                    println!("\nDone. Exiting without injection.");
                    return Ok(());
                }
            }
        }
    }

    println!("\nNo acclient.exe process found");
    println!("\nDone. Exiting without injection.");
    Ok(())
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
            &dll,
            function.as_deref(),
        ),
    }
}
