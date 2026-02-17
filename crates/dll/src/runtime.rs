use std::sync::Once;
use std::time::Duration;

use tokio::runtime::Runtime;

static mut RT: Option<Runtime> = None;
static RT_INIT: Once = Once::new();

#[allow(static_mut_refs)]
pub fn ensure_runtime() -> &'static Runtime {
    unsafe {
        RT_INIT.call_once(|| {
            RT = Some(Runtime::new().expect("Failed to create Tokio runtime"));
        });
        RT.as_ref().unwrap()
    }
}

#[allow(static_mut_refs)]
pub fn shutdown_runtime() -> anyhow::Result<()> {
    unsafe {
        if let Some(rt) = RT.take() {
            rt.shutdown_timeout(Duration::from_millis(100));
        }
    }

    Ok(())
}
