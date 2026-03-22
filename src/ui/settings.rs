//! 設定スクリーン
//!
//! # レイアウト（上から）
//! 1. 目標設定（カロリー / 体重 / PFC）
//! 2. プロフィール（性別・年齢・身長）＋ BMI 参考値
//! 3. 保存ボタン

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::app::App;
use crate::ui::{card, primary_button, ACCENT, C_COLOR, F_COLOR, KCAL_COLOR, MUTED, P_COLOR, TEXT_DARK};

pub fn draw(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        draw_goals_card(app, ui);
        ui.add_space(8.0);

        draw_profile_card(app, ui);
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() / 2.0 - 60.0);
            if primary_button(ui, "  設定を保存  ").clicked() {
                app.save_settings();
            }
        });

        ui.add_space(8.0);
    });
}

// ── 目標設定 ──────────────────────────────────────────────────────────────────

fn draw_goals_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.label(RichText::new("🎯 目標設定").size(14.0).strong().color(TEXT_DARK));
        ui.add_space(10.0);

        // 目標カロリー
        setting_row(ui, "目標カロリー", KCAL_COLOR, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.target_kcal_input)
                    .desired_width(90.0)
                    .hint_text("2000"),
            );
            ui.label(RichText::new("kcal / 日").color(MUTED).size(12.0));
        });
        ui.add_space(6.0);

        // 目標体重
        setting_row(ui, "目標体重", Color32::from_rgb(92, 98, 104), |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.target_weight_input)
                    .desired_width(90.0)
                    .hint_text("60.0"),
            );
            ui.label(RichText::new("kg").color(MUTED).size(12.0));
        });
        ui.add_space(10.0);

        // 目標 PFC
        ui.label(RichText::new("目標 PFC（1 日あたり）").size(12.0).color(MUTED));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            pfc_field(ui, "P タンパク質", P_COLOR, &mut app.target_p_input, "150");
            ui.add_space(8.0);
            pfc_field(ui, "F 脂質",       F_COLOR, &mut app.target_f_input, "60");
            ui.add_space(8.0);
            pfc_field(ui, "C 炭水化物",   C_COLOR, &mut app.target_c_input, "250");
        });
    });
}

fn setting_row(
    ui: &mut Ui,
    label: &str,
    color: Color32,
    add_fields: impl FnOnce(&mut Ui),
) {
    ui.horizontal(|ui| {
        ui.colored_label(color, "●");
        ui.label(RichText::new(label).size(13.0).color(TEXT_DARK).strong());
        ui.add_space(8.0);
        add_fields(ui);
    });
}

fn pfc_field(ui: &mut Ui, label: &str, color: Color32, val: &mut String, hint: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(11.0).color(color).strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(val)
                    .desired_width(72.0)
                    .hint_text(hint),
            );
            ui.label(RichText::new("g").color(MUTED).size(11.0));
        });
    });
}

// ── プロフィール ──────────────────────────────────────────────────────────────

fn draw_profile_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.label(RichText::new("👤 プロフィール").size(14.0).strong().color(TEXT_DARK));
        ui.add_space(10.0);

        // 性別
        ui.horizontal(|ui| {
            ui.label(RichText::new("性別").size(13.0).color(TEXT_DARK).strong());
            ui.add_space(8.0);
            for gender in ["男性", "女性", "その他"] {
                let is_active = app.user_gender == gender;
                let btn = ui.add(
                    egui::Button::new(
                        RichText::new(gender)
                            .size(12.0)
                            .color(if is_active { Color32::WHITE } else { MUTED }),
                    )
                    .fill(if is_active { ACCENT } else { Color32::from_rgb(232, 235, 238) })
                    .rounding(egui::Rounding::same(20.0)),
                );
                if btn.clicked() {
                    app.user_gender = gender.to_string();
                }
            }
        });
        ui.add_space(8.0);

        // 年齢・身長
        ui.horizontal(|ui| {
            ui.label(RichText::new("年齢").size(13.0).color(TEXT_DARK).strong());
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.user_age_input)
                    .desired_width(60.0)
                    .hint_text("25"),
            );
            ui.label(RichText::new("歳").color(MUTED).size(12.0));

            ui.add_space(16.0);

            ui.label(RichText::new("身長").size(13.0).color(TEXT_DARK).strong());
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.user_height_input)
                    .desired_width(60.0)
                    .hint_text("170"),
            );
            ui.label(RichText::new("cm").color(MUTED).size(12.0));
        });
        ui.add_space(10.0);

        // BMI 参考値
        draw_bmi_reference(app, ui);
    });
}

fn draw_bmi_reference(app: &App, ui: &mut Ui) {
    let height_cm: f64 = app.user_height_input.parse().unwrap_or(0.0);
    let current_weight = app.weight_input_as_f64();

    if height_cm <= 0.0 {
        ui.label(RichText::new("身長を入力すると BMI を計算します").color(MUTED).size(11.0));
        return;
    }

    let h_m = height_cm / 100.0;
    let h2  = h_m * h_m;

    egui::Frame::none()
        .fill(Color32::from_rgb(241, 243, 245))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(RichText::new("📊 BMI 参考値").size(12.0).color(MUTED));
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                // 現在の BMI
                if current_weight > 0.0 {
                    let bmi = current_weight / h2;
                    let (bmi_label, bmi_color) = bmi_category(bmi);
                    ui.label(RichText::new(format!("現在: {:.1}", bmi)).size(13.0).color(TEXT_DARK).strong());
                    ui.label(RichText::new(format!("({})", bmi_label)).size(12.0).color(bmi_color));
                    ui.add_space(12.0);
                }

                // 目標体重の BMI
                if app.target_weight > 0.0 {
                    let bmi_target = app.target_weight / h2;
                    let (label, color) = bmi_category(bmi_target);
                    ui.label(
                        RichText::new(format!("目標体重 ({:.1}kg): BMI {:.1} ({})", app.target_weight, bmi_target, label))
                            .size(12.0)
                            .color(color),
                    );
                }
            });

            ui.add_space(6.0);

            // 標準体重
            let std_weight = 22.0 * h2;
            ui.label(
                RichText::new(format!(
                    "標準体重（BMI 22）: {:.1} kg  ／  適正範囲: {:.1} 〜 {:.1} kg",
                    std_weight,
                    18.5 * h2,
                    24.9 * h2,
                ))
                .size(11.0)
                .color(MUTED),
            );
        });
}

fn bmi_category(bmi: f64) -> (&'static str, Color32) {
    match bmi {
        b if b < 18.5 => ("低体重", Color32::from_rgb(146, 152, 158)),
        b if b < 25.0 => ("普通体重", Color32::from_rgb(94, 100, 106)),
        b if b < 30.0 => ("過体重", Color32::from_rgb(118, 124, 130)),
        _             => ("肥満", Color32::from_rgb(70, 74, 78)),
    }
}

// App に weight_input_as_f64 ヘルパーが必要なため、ここで trait 外で呼ぶ
impl App {
    pub fn weight_input_as_f64(&self) -> f64 {
        // 今日の体重フォーム値 or 0
        self.weight_input.trim().parse().unwrap_or(0.0)
    }
}
