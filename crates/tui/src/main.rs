use std::io;

use app::App;

pub mod app;
pub mod tabs;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();

    app_result
}
