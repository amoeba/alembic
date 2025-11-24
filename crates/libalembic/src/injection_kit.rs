#![cfg(target_os = "windows")]

use dll_syringe::{
    error::EjectError,
    process::{BorrowedProcessModule, OwnedProcess},
    rpc::PayloadRpcError,
    Syringe,
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

    pub fn call_startup(&mut self) -> Result<(), PayloadRpcError> {
        println!("call_startup");

        let remote_startup = unsafe {
            self.syringe
                .get_payload_procedure::<fn()>(self.payload.unwrap_unchecked(), "dll_startup")
        }
        .unwrap()
        .unwrap();

        println!("Calling remote startup...");
        Ok(remote_startup.call()?)
    }

    pub fn call_shutdown(&mut self) -> Result<(), PayloadRpcError> {
        println!("call_shutdown");

        let remote_shutdown = unsafe {
            self.syringe
                .get_payload_procedure::<fn()>(self.payload.unwrap_unchecked(), "dll_shutdown")
        }
        .unwrap()
        .unwrap();

        println!("Calling remote shutdown...");
        Ok(remote_shutdown.call()?)
    }
}
