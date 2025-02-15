use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use chrono::{DateTime, Local};
use eframe::egui::{self, Response, Ui, Widget};

use crate::backend::{self, Backend};

pub struct News {
    fetch_period: std::time::Duration,
    last_fetched: Option<std::time::SystemTime>,
}

impl News {
    pub fn default() -> Self {
        Self {
            fetch_period: std::time::Duration::from_secs(10),
            last_fetched: None,
        }
    }
}
impl Widget for &mut News {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
          ui.heading("Community Updates");

          if let Some(b) =
            ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend")))
        {
          let backend = b.lock().unwrap();

          if backend.news.is_none() || backend.news.as_ref().unwrap().entries.len() == 0 {
            ui.label("No news to show");
          } else {
            backend.news.as_ref().unwrap().entries.iter().for_each(|entry| {
              let datetime: DateTime<Local> = entry.datetime.into();

              egui::ScrollArea::vertical()
              .auto_shrink(true)
              .max_height(105.0) // Help last line be cut off to indicate
                                            // the need to scroll
              .show(ui, |ui| {
                  ui.label(format!("{}", datetime.format("%Y-%m-%d %H:%M:%S")));
                  let message = "Have any old hard drives laying around that might have old installs of Asheron's Call, decal plugins, screenshots, or other files that might be otherwise lost to time?  Uploading them to us could help us re-create old landmasses and content that you thought you'd never be able to see again. Or are you just interested in seeing what the community has already put together (old and current emulators, dat files, decal plugins, high res scans of posters and much much more), check out the Asheron’s Call Community Preservation Project today!\n\nBrought to you by Immortalbob and myself.";

                  ui.label(message)
              });
            });

          }
        } else {
          ui.label("Failed to get backend.");
        }

        })
        .response
    }
}
