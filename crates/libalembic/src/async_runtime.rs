use std::sync::Arc;

use tokio::runtime::Runtime;

pub struct AsyncRuntime {
    runtime: Arc<Runtime>,
}

impl AsyncRuntime {
    pub fn new() -> anyhow::Result<Self> {
        let runtime = Arc::new(Runtime::new()?);
        Ok(Self { runtime })
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let runtime = self.runtime.clone();
        runtime.spawn(future);
    }

    pub fn shutdown(self) {
        // Drop the Arc, which will shut down the runtime if it's the last reference
        drop(self.runtime);
    }
}
