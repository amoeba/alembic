use anyhow::Result;
use clap::Parser;
use std::process::Command;

#[derive(Parser)]
#[command(name = "cork")]
#[command(about = "AC client launcher utility for Alembic", long_about = None)]
struct Args {
    /// Path to the AC client executable
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

fn main() -> Result<()> {
    let args = Args::parse();

    launch_client(
        &args.client,
        &args.hostname,
        &args.port,
        &args.account,
        &args.password,
    )?;

    Ok(())
}
