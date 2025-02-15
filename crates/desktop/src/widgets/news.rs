use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use chrono::{DateTime, Local};
use eframe::egui::{self, Response, Ui, Widget};

use crate::{backend::Backend, FetchRequest};

pub struct News {
    num_retry_attempts: usize,
    max_retry_attempts: usize,
    retry_interval: std::time::Duration,
    last_fetched: Option<std::time::SystemTime>,
    is_resolved: bool,
}

impl News {
    pub fn default() -> Self {
        Self {
            num_retry_attempts: 0,
            max_retry_attempts: 3,
            retry_interval: std::time::Duration::from_secs(1),
            last_fetched: None,
            is_resolved: false,
        }
    }
}
impl Widget for &mut News {
    fn ui(self, ui: &mut Ui) -> Response {
        // Handle news update signalling
        if !self.is_resolved {
            // First we check if the were were resolved externally
            if let Some(b) = ui.data_mut(|data| {
                data.get_persisted::<Arc<Mutex<Backend>>>(egui::Id::new("backend"))
            }) {
                let backend = b.lock().unwrap();

                if backend.news.is_some() {
                    self.is_resolved = true;
                }
            }

            // If we're due to check, check
            let retry_not_exhausted = self.max_retry_attempts > self.num_retry_attempts;
            let due_for_retry = self.last_fetched.is_none()
                || self.last_fetched.unwrap() + self.retry_interval < SystemTime::now();

            if retry_not_exhausted && due_for_retry {
                self.last_fetched = Some(SystemTime::now());

                if let Some(sender) = ui.data_mut(|data| {
                    data.get_persisted::<std::sync::mpsc::Sender<FetchRequest>>(egui::Id::new(
                        "background_fetch_sender",
                    ))
                }) {
                    sender
                        .send(FetchRequest::FetchNews)
                        .expect("Failed to send fetch request");
                }

                self.num_retry_attempts += 1;
            }
        }

        // Regular UI code continues here
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
                            let datetime: DateTime<Local> = entry.created_at.into();

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
