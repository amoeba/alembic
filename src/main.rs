pub mod inject;
pub mod launch;

use std::sync::mpsc::channel;

use anyhow::{bail, Error};
use launch::Launcher;

fn main() -> Result<(), anyhow::Error> {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // TODO: Pull config from somewhere
    // TODO: Make following code use that config

    let mut launcher = Launcher::new();

    launcher.find_or_launch()?;
    launcher.inject()?;

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("ctrl+c received...");

    // TODO: Eject
    // TODO: Not working quite right
    // launcher.eject();

    println!("Exiting.");

    Ok(())
}
