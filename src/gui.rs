use eframe::egui::{self, Context, Ui};

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Alembic",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::default())),
    )
}

struct Panels {}

impl Panels {
    fn main(ui: &mut Ui) {
        if ui.add(egui::Button::new("Test")).clicked() {
            println!("clicked");
        }
    }

    fn developer(ui: &mut Ui) {
        if ui.add(egui::Button::new("Developer!")).clicked() {
            println!("clicked");
        }
    }
}

#[derive(PartialEq)]
enum Tab {
    Main,
    Developer,
}

struct MyApp {
    current_tab: Tab,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            current_tab: Tab::Main,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(egui::Button::new("Testing")).clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui: &mut egui::Ui| {
                    if ui.add(egui::Button::new("About")).clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
                ui.selectable_value(&mut self.current_tab, Tab::Developer, "Developer");
            });

            ui.separator();

            match self.current_tab {
                Tab::Main => Panels::main(ui),
                Tab::Developer => Panels::developer(ui),
            }
        });
    }
}
