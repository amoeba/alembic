use anyhow::Result;
use clap::Parser;
use std::process::Command;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
use dll_syringe::process::OwnedProcess;

#[derive(Parser)]
#[command(name = "cork")]
#[command(about = "AC client launcher utility for Alembic", long_about = None)]
struct Args {
    /// Path to DLL to inject (Windows format, e.g., C:\\Program Files (x86)\\Decal\\Inject.dll)
    #[arg(long)]
    dll: Option<String>,
}

#[cfg(all(target_os = "windows", target_env = "msvc"))]
fn launch_client(
    client_path: &str,
    hostname: &str,
    port: &str,
    account: &str,
    password: &str,
) -> Result<()> {
    println!("Launching AC client...");
    println!("  Client: {}", client_path);
    println!("  Server: {}:{}", hostname, port);
    println!("  Account: {}", account);

    let mut cmd = Command::new(client_path);
    cmd.arg("-h").arg(hostname);
    cmd.arg("-p").arg(port);
    cmd.arg("-a").arg(account);
    cmd.arg("-w").arg(password);

    let status = cmd.status()?;

    if status.success() {
        println!("Client launched successfully");
    } else {
        anyhow::bail!("Client exited with status: {}", status);
    }

    Ok(())
}

#[cfg(not(all(target_os = "windows", target_env = "msvc")))]
fn launch_client(
    _client_path: &str,
    _hostname: &str,
    _port: &str,
    _account: &str,
    _password: &str,
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
    let output = Command::new("tasklist.exe")
        .output()?;

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

    let dll_path = match args.dll {
        Some(path) => path,
        None => {
            anyhow::bail!("No DLL path provided. Use --dll to specify the DLL to inject.");
        }
    };

    println!("DLL path: {}", dll_path);
    println!("Searching for running acclient.exe to inject into...");
    println!();

    // TODO: Actually perform injection with the DLL
    find_acclient_windows()
}
