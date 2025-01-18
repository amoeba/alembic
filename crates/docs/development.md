# Development

## Backend

This is how I'm currently accessing the Backend from inside a Widget:

```rust
impl Widget for &mut DeveloperLogsTab {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.group(|ui| {
              // Fill this in
            })
            .response
        } else {
            ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
```
