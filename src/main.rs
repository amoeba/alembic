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

    // Old syringe code for testing eject
    // let target_process =
    //     dll_syringe::process::OwnedProcess::find_first_by_name("acclient.exe").unwrap();
    // let syringe = dll_syringe::Syringe::for_process(target_process);
    // let injected_payload = syringe
    //     .inject("target\\i686-pc-windows-msvc\\debug\\alembic.dll")
    //     .unwrap();
    // syringe.eject(injected_payload).unwrap();

    // Block until Ctrl+C
    rx.recv().expect("Could not receive from channel.");
    println!("ctrl+c received...");

    // TODO: Eject
    // TODO: Not working quite right
    launcher.eject();

    println!("Exiting.");

    Ok(())
}
