#[derive(Debug)]
pub struct NoopLauncher {}

trait Launcher {
    fn new() -> Self;
    fn launch(&self);
}

impl Launcher for NoopLauncher {
    pub fn new() -> Self {
        NoopLauncher {}
    }

    pub fn launch(&self) {}
}
