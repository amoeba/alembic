use std::{
    sync::{Arc, Mutex},
    time::SystemTime,
};

use chrono::{DateTime, Local};
use eframe::egui::{Id, Response, RichText, ScrollArea, Ui, Widget};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::{
    backend::Backend,
    fetching::{BackgroundFetchRequest, FetchWrapper},
};

pub struct News {
    num_retry_attempts: usize,
    max_retry_attempts: usize,
    retry_interval: std::time::Duration,
    last_fetched: Option<std::time::SystemTime>,
    is_resolved: bool,
    commonmark_cache: CommonMarkCache,
}

impl News {
    pub fn default() -> Self {
        Self {
            num_retry_attempts: 0,
            max_retry_attempts: 3,
            retry_interval: std::time::Duration::from_secs(60),
            last_fetched: None,
            is_resolved: false,
            commonmark_cache: CommonMarkCache::default(),
        }
    }
}
impl Widget for &mut News {
    fn ui(self, ui: &mut Ui) -> Response {
        // Handle news update signalling
        if !self.is_resolved {
            // First we check if the were were resolved externally
            if let Some(b) =
                ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(Id::new("backend")))
            {
                let backend = b.lock().unwrap();

                if matches!(&backend.news, FetchWrapper::Success(_news)) {
                    self.is_resolved = true;
                }
            }

            // If we're due to check, check
            let retry_not_exhausted = self.max_retry_attempts > self.num_retry_attempts;
            let due_for_retry = self.last_fetched.is_none()
                || self.last_fetched.unwrap() + self.retry_interval < SystemTime::now();

            if retry_not_exhausted && due_for_retry {
                self.last_fetched = Some(SystemTime::now());

                if let Some(sender) = ui.data_mut(|data| -> _ {
                    data.get_persisted::<std::sync::mpsc::Sender<BackgroundFetchRequest>>(Id::new(
                        "background_fetch_sender",
                    ))
                }) {
                    sender
                        .send(BackgroundFetchRequest::FetchNews)
                        .expect("Failed to send fetch request");
                }

                self.num_retry_attempts += 1;
            }
        }

        // Regular UI code continues here
        ui.vertical(|ui| {
            ui.heading("Community Updates");
            ui.add_space(8.0);
            if let Some(b) =
                ui.data_mut(|data| data.get_persisted::<Arc<Mutex<Backend>>>(Id::new("backend")))
            {
                let backend = b.lock().unwrap();

                match &backend.news {
                    FetchWrapper::NotStarted => {
                        ui.label("Not yet fetched.");
                    }
                    FetchWrapper::Started => {
                        ui.label("Fetching...");
                    }
                    FetchWrapper::Retrying(_) => {
                        ui.label("Retrying...");
                    }
                    FetchWrapper::Success(news) => {
                        if news.entries.is_empty() {
                            ui.label("No news to show");
                        } else {
                            ScrollArea::vertical().auto_shrink(true).show(ui, |ui| {
                                news.entries.iter().for_each(|entry| {
                                    let datetime: DateTime<Local> = entry.created_at.into();

                                    ui.label(RichText::new(entry.subject.clone()).strong());
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(format!(
                                            "Posted by {} on {}",
                                            entry.author,
                                            datetime.format("%Y-%m-%d %H:%M:%S")
                                        ));
                                        ui.hyperlink_to(
                                            "Source".to_string(),
                                            entry.source_url.clone(),
                                        );
                                    });
                                    ui.add_space(4.0);
                                    CommonMarkViewer::new().show(
                                        ui,
                                        &mut self.commonmark_cache,
                                        &entry.body,
                                    );
                                    ui.separator();
                                    ui.add_space(8.0);
                                });
                            });
                        }
                    }
                    FetchWrapper::Failed(error) => {
                        ui.label(format!("Error fetching news: {}", error));
                    }
                }
            } else {
                ui.label("Failed to get backend.");
            }
        })
        .response
    }
}
