#![cfg(all(target_os = "windows", target_env = "msvc"))]

use dll_syringe::{
    error::EjectError, process::BorrowedProcessModule, process::OwnedProcess, Syringe,
};

#[derive(Debug)]
pub struct InjectionKit {
    syringe: Syringe,                                // Owns the Syringe
    payload: Option<BorrowedProcessModule<'static>>, // Stores the injected payload
}

impl InjectionKit {
    pub fn new(target_process: OwnedProcess) -> Self {
        let syringe = Syringe::for_process(target_process);

        InjectionKit {
            syringe,
            payload: None,
        }
    }

    pub fn inject(&mut self, dll_path: &str) -> Result<(), anyhow::Error> {
        let payload = self.syringe.inject(dll_path)?;
        self.payload = Some(unsafe { std::mem::transmute(payload) });

        Ok(())
    }

    pub fn eject(&mut self) -> Result<(), EjectError> {
        if let Some(payload) = self.payload.take() {
            self.syringe.eject(payload)?;
        }

        Ok(())
    }
}
