use std::sync::mpsc::channel;

use dll_syringe::{
    process::{OwnedProcess, Process},
    Syringe,
};

fn main() {
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    let target_process;

    match OwnedProcess::find_first_by_name("WindowsProject1") {
        Some(value) => {
            target_process = value;
        }
        None => {
            println!("Couldn't find process by name with error: Exiting.");
            return;
        }
    }

    // debugging
    if OwnedProcess::is_x86(&target_process).unwrap() {
        println!("target is 32-bit");
    } else {
        println!("target is not 32-bit");
    }

    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe
        .inject("target\\i686-pc-windows-msvc\\debug\\alembic.dll")
        .unwrap();

    println!("DLL injected successfully!");

    rx.recv().expect("Could not receive from channel.");
    println!("Got it! Ejecting and exiting...");

    syringe.eject(injected_payload).unwrap();
}
