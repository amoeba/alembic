# Development

## Widgets

For uniformity, I'm encapsulating resuable bits of UI using the `eframe::egui::Widget` trait.

```rust
struct SomeWidget {}
impl Widget for &mut SomeWidget {
    fn ui(self, ui: &mut Ui) -> Response {
      // Code here
    }
}
```

## Backend

Cross-component data is persisted using egui [Context](https://docs.rs/egui/latest/egui/struct.Context.html).
This is how I'm currently accessing the Backend from inside a Widget:

```rust
impl Widget for &mut SomeWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        if let Some(backend) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
            ui.group(|ui| {
              // Fill this in with your UI
            })
            .response
        } else {
            ui.group(|ui| centered_text(ui, "Failed to reach application backend."))
                .response
        }
    }
}
```

Note: I don't yet have a nice pattern for a getting a read-only lock on the backend so all code uses `.data_mut`.
If you have a good method, please let me know.
