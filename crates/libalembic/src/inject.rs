#![cfg(all(target_os = "windows", target_env = "msvc"))]

use dll_syringe::{
    error::EjectError,
    process::BorrowedProcessModule,
    process::OwnedProcess,
    rpc::PayloadRpcError,
    Syringe,
};

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
        // SAFETY: The BorrowedProcessModule's lifetime is tied to the Syringe, which lives
        // in the same struct. Rust drops fields in declaration order (syringe before payload),
        // but we always take() the payload in eject() before the struct is dropped. The
        // transmute to 'static is safe as long as the module is not used after the syringe
        // is dropped.
        self.payload = Some(unsafe { std::mem::transmute(payload) });

        Ok(())
    }

    pub fn call_startup(&mut self) -> Result<(), PayloadRpcError> {
        let remote_startup = unsafe {
            self.syringe
                .get_payload_procedure::<fn()>(self.payload.unwrap_unchecked(), "dll_startup")
        }
        .unwrap()
        .unwrap();
        Ok(remote_startup.call()?)
    }

    pub fn call_shutdown(&mut self) -> Result<(), PayloadRpcError> {
        let remote_shutdown = unsafe {
            self.syringe
                .get_payload_procedure::<fn()>(self.payload.unwrap_unchecked(), "dll_shutdown")
        }
        .unwrap()
        .unwrap();
        Ok(remote_shutdown.call()?)
    }

    pub fn eject(&mut self) -> Result<(), EjectError> {
        if let Some(payload) = self.payload.take() {
            self.syringe.eject(payload)?;
        }

        Ok(())
    }
}
