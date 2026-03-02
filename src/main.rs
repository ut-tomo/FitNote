mod app;
mod db;
mod domain;
mod ui;

use app::App;
use db::Db;

fn main() -> eframe::Result {
    let db = Db::open("app.db").expect("Failed to open database");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Diet Tracker",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(db)))),
    )
}
