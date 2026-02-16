use eframe::egui::{FontFamily, Grid, Response, RichText, ScrollArea, Ui, Vec2, Widget};

pub struct BinaryViewer {
    id: String,
    data: Vec<u8>,
    width: usize,
}

impl BinaryViewer {
    pub fn new(id: String, data: Vec<u8>) -> Self {
        Self { id, data, width: 8 }
    }
}

impl Widget for &mut BinaryViewer {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.group(|ui| {
            ScrollArea::vertical().show(ui, |ui| {
                Grid::new(format!("{}container", self.id))
                    .num_columns(2)
                    .show(ui, |ui| {
                        Grid::new(format!("{}hex", self.id))
                            .num_columns(self.width)
                            .spacing(Vec2::ZERO)
                            .show(ui, |ui| {
                                for i in 0..self.data.len() {
                                    ui.label(
                                        RichText::new(format!("{:02X}", self.data[i]))
                                            .family(FontFamily::Monospace),
                                    );

                                    if (i + 1) % self.width == 0 {
                                        ui.end_row();
                                    }
                                }
                            });

                        Grid::new(format!("{}ascii", self.id))
                            .num_columns(self.width)
                            .spacing(Vec2::ZERO)
                            .min_col_width(8.0)
                            .show(ui, |ui| {
                                for i in 0..self.data.len() {
                                    if self.data[i].is_ascii() {
                                        ui.label(format!("{}", self.data[i] as char));
                                    } else {
                                        ui.label(".");
                                    }

                                    if (i + 1) % self.width == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
            });
        })
        .response
    }
}
