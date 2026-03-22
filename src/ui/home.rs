//! 今日スクリーン
//!
//! # レイアウト（上から）
//! 1. 日付 ＋ 体重入力
//! 2. 栄養サマリカード（kcal/P/F/C プログレスバー）
//! 3. 食事記録カード（スロット選択 → 食材グリッド → 量入力 → 追加）
//! 4. 今日の食品ログ（スロット別）

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::app::App;
use crate::domain::Slot;
use crate::ui::{card, nutrient_bar, primary_button, ACCENT, C_COLOR, F_COLOR, KCAL_COLOR, MUTED, P_COLOR, TEXT_DARK};

pub fn draw(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        // ─ 上段: 日付 ＋ 体重（横並び） ─
        ui.horizontal(|ui| {
            ui.label(RichText::new(&app.today).color(MUTED).size(12.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_weight_inline(app, ui);
            });
        });
        ui.add_space(8.0);

        draw_summary_card(app, ui);
        ui.add_space(8.0);

        draw_log_card(app, ui);
        ui.add_space(8.0);

        draw_food_log(app, ui);
        ui.add_space(8.0);
    });
}

// ── 体重（インライン） ────────────────────────────────────────────────────────

fn draw_weight_inline(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.weight_input)
                .desired_width(70.0)
                .hint_text("体重"),
        );
        ui.label(RichText::new("kg").color(MUTED).size(12.0));
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if enter || primary_button(ui, "保存").clicked() {
            app.save_weight();
        }
    });
}

// ── 栄養サマリ ────────────────────────────────────────────────────────────────

fn draw_summary_card(app: &App, ui: &mut Ui) {
    let s      = app.cached_summary;
    let target = app.target_kcal;

    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("今日の摂取").size(13.0).strong().color(TEXT_DARK));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let ratio = s.kcal / target;
                let kcal_color = match ratio {
                    r if r > 1.1 => Color32::from_rgb(220, 80, 60),
                    r if r > 0.9 => KCAL_COLOR,
                    _            => MUTED,
                };
                ui.label(
                    RichText::new(format!("{} / {:.0} kcal", s.kcal as i32, target))
                        .color(kcal_color)
                        .size(13.0)
                        .strong(),
                );
            });
        });
        ui.add_space(6.0);
        nutrient_bar(ui, "kcal", s.kcal, target,        KCAL_COLOR);
        nutrient_bar(ui, "P   ", s.p,    app.target_p,  P_COLOR);
        nutrient_bar(ui, "F   ", s.f,    app.target_f,  F_COLOR);
        nutrient_bar(ui, "C   ", s.c,    app.target_c,  C_COLOR);
    });
}

// ── 食事記録カード ────────────────────────────────────────────────────────────

fn draw_log_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.label(RichText::new("食事を記録").size(13.0).strong().color(TEXT_DARK));
        ui.add_space(8.0);

        draw_slot_selector(app, ui);
        ui.add_space(10.0);

        draw_food_grid(app, ui);

        if app.selected_food.is_some() {
            ui.add_space(8.0);
            draw_amount_input(app, ui);
        }
    });
}

fn draw_slot_selector(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        for slot in Slot::all() {
            let is_active = app.active_slot == slot;
            let btn = ui.add(
                egui::Button::new(
                    RichText::new(slot.label())
                        .size(12.0)
                        .color(if is_active { Color32::WHITE } else { MUTED }),
                )
                .fill(if is_active {
                    ACCENT
                } else {
                    Color32::from_rgb(232, 235, 238)
                })
                .rounding(egui::Rounding::same(20.0)),
            );
            if btn.clicked() {
                app.active_slot = slot;
            }
        }
    });
}

/// 食材グリッド：検索ボックス ＋ カード型ボタンをラップして表示。
fn draw_food_grid(app: &mut App, ui: &mut Ui) {
    // 検索ボックス
    ui.horizontal(|ui| {
        ui.label(RichText::new("🔍").size(13.0));
        ui.add(
            egui::TextEdit::singleline(&mut app.food_search)
                .desired_width(200.0)
                .hint_text("食材を検索..."),
        );
        if !app.food_search.is_empty() {
            if ui.small_button("✕").clicked() {
                app.food_search.clear();
                app.selected_food = None;
            }
        }
    });
    ui.add_space(6.0);

    let matches: Vec<_> = app.filtered_foods().into_iter().cloned().collect();

    if matches.is_empty() {
        if !app.food_search.is_empty() {
            ui.label(RichText::new("該当する食材がありません").color(MUTED).size(11.0));
        } else if app.food_list.is_empty() {
            ui.label(
                RichText::new("「食品」タブで食材を登録してください")
                    .color(MUTED)
                    .size(11.0),
            );
        } else {
            ui.label(RichText::new("食材名を入力して検索してください").color(MUTED).size(11.0));
        }
        return;
    }

    let selected_id = app.selected_food.as_ref().map(|f| f.id);
    let mut pick = None;

    // 最大表示件数（全件はグリッドが大きくなりすぎるため）
    let display_foods: Vec<_> = matches.iter().take(40).collect();

    ScrollArea::vertical()
        .id_salt("food_grid_scroll")
        .max_height(200.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for food in &display_foods {
                    let is_selected = selected_id == Some(food.id);

                    let (bg, text_color) = if is_selected {
                        (ACCENT, Color32::WHITE)
                    } else {
                        (Color32::from_rgb(243, 244, 246), TEXT_DARK)
                    };

                    let btn = egui::Frame::none()
                        .fill(bg)
                        .rounding(egui::Rounding::same(10.0))
                        .stroke(egui::Stroke::new(
                            if is_selected { 0.0 } else { 0.8 },
                            Color32::from_rgb(213, 217, 221),
                        ))
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_min_width(76.0);
                            ui.set_max_width(100.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(&food.name)
                                        .size(12.0)
                                        .color(text_color)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(format!("{:.0}kcal", food.kcal_per_unit))
                                        .size(10.0)
                                        .color(if is_selected {
                                            Color32::from_rgb(229, 232, 235)
                                        } else {
                                            KCAL_COLOR
                                        }),
                                );
                                // 単位あたりのPFC（コンパクト表示）
                                ui.label(
                                    RichText::new(format!(
                                        "P{:.0}/F{:.0}/C{:.0}",
                                        food.p_per_unit, food.f_per_unit, food.c_per_unit
                                    ))
                                    .size(9.0)
                                    .color(if is_selected {
                                        Color32::from_rgb(212, 216, 220)
                                    } else {
                                        MUTED
                                    }),
                                );
                            });
                        });

                    if btn.response.interact(egui::Sense::click()).clicked() {
                        pick = Some((*food).clone());
                    }
                }
            });
        });

    if let Some(f) = pick {
        app.selected_food = Some(f);
        app.amount_input = "1".into();
    }

    if matches.len() > 40 {
        ui.label(
            RichText::new(format!("他 {} 件（検索で絞り込んでください）", matches.len() - 40))
                .color(MUTED)
                .size(10.0),
        );
    }
}

fn draw_amount_input(app: &mut App, ui: &mut Ui) {
    let unit_label = app.selected_food.as_ref().map(|f| f.unit.as_str()).unwrap_or("g");

    egui::Frame::none()
        .fill(Color32::from_rgb(240, 242, 244))
        .rounding(egui::Rounding::same(8.0))
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(f) = &app.selected_food {
                    ui.label(RichText::new(&f.name).color(ACCENT).size(13.0).strong());
                    ui.label(RichText::new("を").color(MUTED).size(12.0));
                }
                ui.add(egui::TextEdit::singleline(&mut app.amount_input).desired_width(60.0));
                ui.label(RichText::new(unit_label).color(MUTED).size(12.0));
                ui.label(RichText::new("食べた").color(MUTED).size(12.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if primary_button(ui, "追加 ✓").clicked() {
                        app.log_meal_item();
                    }
                });
            });
        });
}

// ── 今日の食品ログ ────────────────────────────────────────────────────────────

fn draw_food_log(app: &mut App, ui: &mut Ui) {
    let items = app.today_items.clone();
    let mut delete_id: Option<i64> = None;

    if items.is_empty() {
        return;
    }

    for slot in Slot::all() {
        let slot_items: Vec<_> = items.iter().filter(|(s, _)| *s == slot).map(|(_, i)| i).collect();
        if slot_items.is_empty() { continue; }

        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(slot.label()).size(13.0).color(ACCENT).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let slot_kcal: f64 = slot_items.iter().map(|i| i.kcal).sum();
                    ui.label(RichText::new(format!("{:.0} kcal", slot_kcal)).size(11.0).color(KCAL_COLOR));
                });
            });
            ui.add_space(4.0);

            for item in &slot_items {
                egui::Frame::none()
                    .fill(Color32::from_rgb(252, 248, 255))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&item.food_name).size(12.0).color(TEXT_DARK));
                            ui.label(
                                RichText::new(format!("{} {}", item.amount, item.unit.as_str()))
                                    .size(11.0)
                                    .color(MUTED),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("✕").size(10.0).color(MUTED))
                                            .fill(Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::NONE),
                                    )
                                    .clicked()
                                {
                                    delete_id = Some(item.id);
                                }
                                ui.label(
                                    RichText::new(format!("{:.0} kcal", item.kcal))
                                        .size(11.0)
                                        .color(KCAL_COLOR),
                                );
                            });
                        });
                    });
                ui.add_space(2.0);
            }
        });
        ui.add_space(4.0);
    }

    if let Some(id) = delete_id {
        app.delete_meal_item(id);
    }
}
