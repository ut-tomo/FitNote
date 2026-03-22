//! 食品管理スクリーン
//!
//! ユーザーが食品マスタを自由に登録・編集・削除できる。
//! 追加フォームと食品一覧を縦に並べたシンプルなレイアウト。
//!
//! # レイアウト（上から）
//! 1. 追加 / 編集フォームカード
//! 2. 食品一覧カード（検索フィルター付き）

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::app::App;
use crate::domain::{FoodDraft, Unit};
use crate::ui::{card, primary_button, MUTED, TEXT_DARK};

/// 食品管理スクリーン全体を描画する。
pub fn draw(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        draw_form_card(app, ui);
        ui.add_space(10.0);

        draw_food_list_card(app, ui);
        ui.add_space(8.0);
    });
}

// ── サブ描画関数 ──────────────────────────────────────────────────────────────

/// 追加フォーム（通常時）または編集フォーム（editing_food が Some のとき）を描画する。
fn draw_form_card(app: &mut App, ui: &mut Ui) {
    let is_editing = app.editing_food.is_some();
    let title = if is_editing { "食品を編集" } else { "新しい食品を追加" };

    card(ui, |ui| {
        ui.label(RichText::new(title).size(13.0).strong().color(TEXT_DARK));
        ui.add_space(6.0);

        // 編集中なら editing_food の draft を、そうでなければ new_food を使う
        let draft: &mut FoodDraft = if let Some((_, ref mut d)) = app.editing_food {
            d
        } else {
            &mut app.new_food
        };

        draw_name_row(ui, draft);
        draw_unit_row(ui, draft);
        draw_nutrition_row(ui, draft);

        ui.add_space(6.0);
        draw_form_buttons(app, ui, is_editing);
    });
}

/// 食品名の入力行。
fn draw_name_row(ui: &mut Ui, draft: &mut FoodDraft) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("名前").color(MUTED).size(12.0).strong());
        ui.add(
            egui::TextEdit::singleline(&mut draft.name).desired_width(200.0),
        );
    });
}

/// 計量単位の ComboBox 選択行。
fn draw_unit_row(ui: &mut Ui, draft: &mut FoodDraft) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("単位").color(MUTED).size(12.0).strong());
        egui::ComboBox::from_id_salt("unit_combo")
            .selected_text(draft.unit.as_str())
            .show_ui(ui, |ui| {
                for unit in Unit::all() {
                    ui.selectable_value(&mut draft.unit, *unit, unit.as_str());
                }
            });
    });
}

/// 栄養素（kcal / P / F / C）の入力フィールド群。
fn draw_nutrition_row(ui: &mut Ui, draft: &mut FoodDraft) {
    ui.label(
        RichText::new(format!("1{} あたり", draft.unit.as_str()))
            .color(MUTED)
            .size(11.0),
    );
    ui.horizontal(|ui| {
        nutrition_field(ui, "kcal", &mut draft.kcal);
        nutrition_field(ui, "P(g)", &mut draft.p);
        nutrition_field(ui, "F(g)", &mut draft.f);
        nutrition_field(ui, "C(g)", &mut draft.c);
    });
}

/// 登録 / 保存 / キャンセルボタン群。
fn draw_form_buttons(app: &mut App, ui: &mut Ui, is_editing: bool) {
    ui.horizontal(|ui| {
        if is_editing {
            if primary_button(ui, "保存").clicked() {
                app.save_food_edit();
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("キャンセル").size(12.0).color(MUTED))
                        .fill(Color32::from_rgb(240, 232, 248))
                        .rounding(egui::Rounding::same(8.0)),
                )
                .clicked()
            {
                app.editing_food = None;
            }
        } else if primary_button(ui, "登録").clicked() {
            app.add_food();
        }
    });
}

/// 登録済み食品の一覧カード（検索フィルター付き）。
fn draw_food_list_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        // ヘッダー：「登録済み食品」ラベル ＋ 検索ボックス
        ui.horizontal(|ui| {
            ui.label(RichText::new("登録済み食品").size(13.0).strong().color(TEXT_DARK));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.food_filter)
                        .desired_width(130.0)
                        .hint_text("検索..."),
                );
            });
        });
        ui.add_space(4.0);

        if app.food_list.is_empty() {
            ui.label(
                RichText::new("食品が登録されていません。上のフォームから追加してください。")
                    .color(MUTED)
                    .size(12.0),
            );
            return;
        }

        // 列ヘッダー
        ui.horizontal(|ui| {
            ui.label(RichText::new("名前").size(11.0).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(48.0); // 編集・削除ボタン分のスペース
                ui.label(RichText::new("kcal / C / F / P").size(11.0).color(MUTED));
            });
        });
        ui.separator();

        // 食品行を描画し、アクション（編集・削除）を後処理で実行する
        let foods: Vec<_> = app.filtered_foods_manage().into_iter().cloned().collect();
        let mut edit_id: Option<i64> = None;
        let mut del_id: Option<i64>  = None;

        for food in &foods {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&food.name).size(12.0).color(TEXT_DARK));
                ui.label(
                    RichText::new(format!("/{}", food.unit.as_str()))
                        .size(10.0)
                        .color(MUTED),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("削除").size(10.0).color(Color32::from_rgb(210, 70, 70)),
                            )
                            .fill(Color32::from_rgb(255, 235, 235))
                            .stroke(egui::Stroke::new(0.5, Color32::from_rgb(230, 180, 180)))
                            .rounding(egui::Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        del_id = Some(food.id);
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("編集").size(10.0).color(MUTED))
                                .fill(Color32::from_rgb(232, 235, 238))
                                .stroke(egui::Stroke::new(0.5, Color32::from_rgb(213, 217, 221)))
                                .rounding(egui::Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        edit_id = Some(food.id);
                    }
                    ui.label(
                        RichText::new(format!(
                            "{:.0}kcal  P{:.0}/F{:.0}/C{:.0}",
                            food.kcal_per_unit,
                            food.p_per_unit,
                            food.f_per_unit,
                            food.c_per_unit,
                        ))
                        .size(11.0)
                        .color(MUTED),
                    );
                });
            });
        }

        // ループ後に借用を解放してからアクション実行
        if let Some(id) = del_id {
            app.delete_food(id);
        }
        if let Some(id) = edit_id {
            if let Some(food) = app.food_list.iter().find(|f| f.id == id) {
                app.editing_food = Some((id, FoodDraft::from_item(food)));
            }
        }
    });
}

// ── ウィジェットヘルパー ──────────────────────────────────────────────────────

/// ラベル付きの小さい数値入力フィールド。
fn nutrition_field(ui: &mut Ui, label: &str, val: &mut String) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(10.0).color(MUTED));
        ui.add(egui::TextEdit::singleline(val).desired_width(55.0));
    });
}

// accent_button は primary_button に統合したため削除。
// 後方互換のためにスタブを残す（未使用警告を抑制）
#[allow(dead_code)]
fn accent_button(ui: &mut Ui, label: &str) -> egui::Response {
    primary_button(ui, label)
}
