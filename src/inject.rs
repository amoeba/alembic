use std::sync::Arc;

use dll_syringe::{error::EjectError, process::BorrowedProcessModule, Syringe};

pub struct InjectionKit<'a> {
    syringe: Arc<Syringe>,
    payload: Option<BorrowedProcessModule<'a>>,
}

impl<'a> InjectionKit<'a> {
    pub fn new(syringe: Syringe) -> Self {
        InjectionKit {
            syringe: Arc::new(syringe),
            payload: None,
        }
    }

    pub fn inject(&'a mut self, assembly_path: &str) {
        self.payload = match self.syringe.inject(assembly_path) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub fn eject(&self) -> Result<(), EjectError> {
        self.syringe
            .eject(self.payload.expect("Eject called without payload."))
    }
}
