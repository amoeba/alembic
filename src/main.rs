use dll_syringe::{process::OwnedProcess, Syringe};

fn main() {
    println!("Hello from main!");

    let target_process = OwnedProcess::find_first_by_name("Notepad").unwrap();
    let syringe = Syringe::for_process(target_process);
    let injected_payload = syringe.inject("target/debug/alembic.dll").unwrap();
    syringe.eject(injected_payload).unwrap();
}
