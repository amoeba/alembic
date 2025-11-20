use std::collections::HashMap;
use std::fmt;
use std::path::Path;

pub trait ClientConfig {
    fn name(&self) -> &str;
    fn client_path(&self) -> &Path;
    fn wrapper_program(&self) -> Option<&Path>;
    fn env(&self) -> &HashMap<String, String>;

    fn install_path(&self) -> &Path {
        self.client_path().parent().unwrap_or_else(|| Path::new(""))
    }

    fn fmt_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Name: {}", self.name())?;
        writeln!(f, "Client path: {}", self.client_path().display())?;

        if let Some(wrapper) = self.wrapper_program() {
            writeln!(f, "Type: Wine")?;
            writeln!(f, "Wrapper program: {}", wrapper.display())?;
        } else {
            writeln!(f, "Type: Windows")?;
        }

        if !self.env().is_empty() {
            writeln!(f)?;
            writeln!(f, "Environment variables:")?;
            for (key, value) in self.env() {
                writeln!(f, "  {}={}", key, value)?;
            }
        }

        Ok(())
    }
}
