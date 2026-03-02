use crate::app::App;
use egui::Ui;

pub fn draw(app: &mut App, ui: &mut Ui) {
    ui.heading("Diet Tracker (local)");
    ui.separator();

    // -- Date --
    ui.label(format!("Date: {}", app.today));
    ui.add_space(8.0);

    // -- Weight --
    ui.horizontal(|ui| {
        ui.label("Weight (kg):");
        ui.text_edit_singleline(&mut app.weight_input);
        if ui.button("Save").clicked() {
            if let Ok(kg) = app.weight_input.trim().parse::<f64>() {
                app.save_weight(kg);
            }
        }
    });
    ui.add_space(8.0);

    // -- Meal buttons --
    ui.label("Log meal:");
    ui.horizontal(|ui| {
        if ui.button("Breakfast").clicked() {
            app.log_breakfast();
        }
        if ui.button("Lunch").clicked() {
            app.log_lunch();
        }
        if ui.button("Dinner").clicked() {
            app.log_dinner();
        }
        if ui.button("Snack").clicked() {
            app.log_snack();
        }
    });
    ui.add_space(8.0);

    // -- Toppings --
    ui.label("Toppings:");
    if app.last_meal_log_id.is_none() {
        ui.label("Log a meal first.");
    } else {
        ui.horizontal(|ui| {
            if ui.button("+Egg x1").clicked() {
                app.add_topping("卵1個", 1.0);
            }
            if ui.button("+Meat miso 70g").clicked() {
                app.add_topping("肉味噌70g", 1.0);
            }
            if ui.button("+Wagashi 200kcal").clicked() {
                app.add_topping("和菓子200kcal", 1.0);
            }
            if ui.button("+Lean beef 200g").clicked() {
                app.add_topping("牛赤身200g", 1.0);
            }
        });
    }
    ui.add_space(12.0);

    // -- Today summary --
    ui.separator();
    ui.label("Today's intake:");
    let s = app.refresh_summary();
    ui.monospace(format!("kcal: {:.0}", s.kcal));
    ui.monospace(format!("P:    {:.0} g", s.p));
    ui.monospace(format!("F:    {:.0} g", s.f));
    ui.monospace(format!("C:    {:.0} g", s.c));

    // -- Status --
    if let Some(msg) = &app.status_msg {
        ui.add_space(8.0);
        ui.label(msg.as_str());
    }
}
