use crate::db::Db;
use crate::domain::{Slot, Summary};
use crate::ui;
use chrono::Local;

pub struct App {
    pub today: String,
    pub weight_input: String,
    pub last_meal_log_id: Option<i64>,
    pub status_msg: Option<String>,
    db: Db,
    cached_summary: Summary,
}

impl App {
    pub fn new(db: Db) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let cached_summary = db.get_today_summary(&today).unwrap_or_default();
        App {
            today,
            weight_input: String::new(),
            last_meal_log_id: None,
            status_msg: None,
            db,
            cached_summary,
        }
    }

    pub fn save_weight(&mut self, kg: f64) {
        match self.db.upsert_weight(&self.today, kg) {
            Ok(()) => self.status_msg = Some(format!("Weight saved: {kg} kg")),
            Err(e) => self.status_msg = Some(format!("Error: {e}")),
        }
    }

    pub fn log_breakfast(&mut self) {
        self.log_template_meal(Slot::Breakfast, "朝: オートミール");
    }

    pub fn log_lunch(&mut self) {
        self.log_template_meal(Slot::Lunch, "昼軸: 米200+鶏むね100");
    }

    pub fn log_dinner(&mut self) {
        self.log_template_meal(Slot::Dinner, "夜軸: 米200+鶏むね100");
    }

    pub fn log_snack(&mut self) {
        match self.db.log_meal_no_template(&self.today, Slot::Snack) {
            Ok(id) => {
                self.last_meal_log_id = Some(id);
                self.invalidate_summary();
                self.status_msg = Some("Snack logged".into());
            }
            Err(e) => self.status_msg = Some(format!("Error: {e}")),
        }
    }

    pub fn add_topping(&mut self, food_name: &str, multiplier: f64) {
        if let Some(meal_id) = self.last_meal_log_id {
            match self.db.add_item_to_meal(meal_id, food_name, multiplier) {
                Ok(()) => {
                    self.invalidate_summary();
                    self.status_msg = Some(format!("Added {food_name}"));
                }
                Err(e) => self.status_msg = Some(format!("Error: {e}")),
            }
        }
    }

    pub fn refresh_summary(&mut self) -> Summary {
        self.cached_summary
    }

    fn log_template_meal(&mut self, slot: Slot, template_name: &str) {
        match self.db.log_meal_template(&self.today, slot, template_name) {
            Ok(id) => {
                self.last_meal_log_id = Some(id);
                self.invalidate_summary();
                self.status_msg = Some(format!("{} logged", slot.as_str()));
            }
            Err(e) => self.status_msg = Some(format!("Error: {e}")),
        }
    }

    fn invalidate_summary(&mut self) {
        self.cached_summary = self.db.get_today_summary(&self.today).unwrap_or_default();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::home::draw(self, ui);
        });
    }
}
