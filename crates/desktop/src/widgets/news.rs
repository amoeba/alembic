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

            if let Some(b) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
            }) {
                let backend = b.lock().unwrap();

                if backend.news.is_none() || backend.news.as_ref().unwrap().entries.len() == 0 {
                    ui.label("No news to show");
                } else {
                    backend
                        .news
                        .as_ref()
                        .unwrap()
                        .entries
                        .iter()
                        .for_each(|entry| {
                            let datetime: DateTime<Local> = entry.datetime.into();

                            egui::ScrollArea::vertical()
                                .auto_shrink(true)
                                .max_height(105.0) // Help last line be cut off to indicate
                                // the need to scroll
                                .show(ui, |ui| {
                                    ui.label(format!("{}", datetime.format("%Y-%m-%d %H:%M:%S")));
                                    ui.label(entry.body.clone())
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
