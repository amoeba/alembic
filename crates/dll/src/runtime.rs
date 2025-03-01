use std::sync::Once;
use std::time::Duration;

use tokio::runtime::Runtime;

static mut rt: Option<Runtime> = None;
static rt_init: Once = Once::new();

#[allow(static_mut_refs)]
pub fn ensure_runtime() -> &'static Runtime {
    unsafe {
        rt_init.call_once(|| {
            rt = Some(Runtime::new().expect("Failed to create Tokio runtime"));
        });
        rt.as_ref().unwrap()
    }
}

#[allow(static_mut_refs)]
pub fn shutdown_runtime() -> anyhow::Result<()> {
    unsafe {
        rt.take()
            .unwrap()
            .shutdown_timeout(Duration::from_millis(100));
    }

    Ok(())
}
