//! 筋トレスクリーン
//!
//! # サブタブ構成
//! 今日のトレーニング | 種目管理 | 履歴 | AI分析

use egui::{Color32, Pos2, RichText, ScrollArea, Stroke, Ui};

use crate::app::{App, TrainingSubScreen};
use crate::domain::{Exercise, ExerciseDraft, TrainingSet};
use crate::ui::{card, primary_button, ACCENT, MUTED, TEXT_DARK};

const MUSCLE_GROUPS: &[&str] = &["", "胸", "背中", "脚", "肩", "腕", "腹", "その他"];

pub fn draw(app: &mut App, ui: &mut Ui) {
    draw_sub_tab_bar(app, ui);
    ui.add_space(4.0);

    match app.training_sub {
        TrainingSubScreen::Today     => draw_today(app, ui),
        TrainingSubScreen::Exercises => draw_exercises(app, ui),
        TrainingSubScreen::History   => draw_history(app, ui),
        TrainingSubScreen::Analysis  => draw_analysis(app, ui),
    }
}

// ── サブタブバー ──────────────────────────────────────────────────────────────

fn draw_sub_tab_bar(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        sub_tab_btn(ui, app, TrainingSubScreen::Today,     "今日");
        sub_tab_btn(ui, app, TrainingSubScreen::Exercises, "種目管理");
        sub_tab_btn(ui, app, TrainingSubScreen::History,   "履歴");
        sub_tab_btn(ui, app, TrainingSubScreen::Analysis,  "AI分析");
    });
    ui.separator();
}

fn sub_tab_btn(ui: &mut Ui, app: &mut App, screen: TrainingSubScreen, label: &str) {
    let is_active = app.training_sub == screen;
    let color = if is_active { ACCENT } else { MUTED };
    let btn = ui.add(
        egui::Button::new(RichText::new(label).size(12.0).color(color))
            .fill(Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE),
    );
    if is_active {
        let r = btn.rect;
        ui.painter().line_segment(
            [egui::pos2(r.left(), r.bottom()), egui::pos2(r.right(), r.bottom())],
            egui::Stroke::new(2.0, ACCENT),
        );
    }
    if btn.clicked() {
        app.training_sub = screen;
    }
}

// ── 今日のトレーニング ────────────────────────────────────────────────────────

fn draw_today(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(2.0);

        // 日付 + メモ
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&app.today).size(12.0).color(MUTED));
                ui.label(RichText::new("メモ:").size(12.0).color(MUTED));
                let memo_resp = ui.add(
                    egui::TextEdit::singleline(&mut app.session_memo)
                        .desired_width(200.0)
                        .hint_text("任意メモ"),
                );
                if memo_resp.lost_focus() {
                    if let Some(id) = app.training_session_id {
                        let memo = app.session_memo.clone();
                        let _ = app.db.update_session_memo(id, &memo);
                    }
                }
            });
        });
        ui.add_space(6.0);

        // 種目選択 + セット入力
        card(ui, |ui| {
            ui.label(RichText::new("種目を選ぶ").size(12.0).strong().color(TEXT_DARK));
            ui.add_space(4.0);

            // 検索ボックス
            ui.add(
                egui::TextEdit::singleline(&mut app.exercise_search)
                    .desired_width(ui.available_width())
                    .hint_text("種目を検索..."),
            );
            ui.add_space(4.0);

            // 種目グリッドボタン
            let query = app.exercise_search.to_lowercase();
            let exercises: Vec<Exercise> = app.exercise_list.iter()
                .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query))
                .cloned()
                .collect();

            if exercises.is_empty() {
                ui.label(RichText::new("種目がありません。「種目管理」タブで追加してください。")
                    .size(11.0).color(MUTED));
            } else {
                let cols = 3usize;
                let rows = (exercises.len() + cols - 1) / cols;
                for row in 0..rows {
                    ui.horizontal(|ui| {
                        for col in 0..cols {
                            let idx = row * cols + col;
                            if idx >= exercises.len() { break; }
                            let ex = &exercises[idx];
                            let is_sel = app.selected_exercise.as_ref().map(|e| e.id) == Some(ex.id);
                            let btn = ui.add(
                                egui::Button::new(
                                    RichText::new(&ex.name)
                                        .size(11.0)
                                        .color(if is_sel { Color32::WHITE } else { TEXT_DARK }),
                                )
                                .fill(if is_sel { ACCENT } else { Color32::from_rgb(240, 235, 250) })
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(80.0, 26.0)),
                            );
                            if btn.clicked() {
                                app.selected_exercise = Some(ex.clone());
                            }
                        }
                    });
                }
            }
        });
        ui.add_space(6.0);

        // セット入力（種目選択済みの場合）
        if let Some(ref ex) = app.selected_exercise.clone() {
            card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("● {} ({})", ex.name, ex.muscle_group))
                            .size(12.0).strong().color(ACCENT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("別の種目").size(11.0).color(MUTED)).clicked() {
                            app.selected_exercise = None;
                        }
                    });
                });
                ui.add_space(4.0);

                // 今の種目のセット一覧
                let sets: Vec<TrainingSet> = app.session_sets.iter()
                    .filter(|s| s.exercise_id == ex.id)
                    .cloned()
                    .collect();

                let mut delete_id: Option<i64> = None;
                for s in &sets {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!(
                            "Set {}: {}kg × {}reps",
                            s.set_number, s.weight_kg, s.reps
                        )).size(11.0).color(TEXT_DARK));
                        if ui.small_button("✕").clicked() {
                            delete_id = Some(s.id);
                        }
                    });
                }
                if let Some(id) = delete_id {
                    app.delete_set(id);
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("重量:").size(12.0).color(MUTED));
                    ui.add(
                        egui::TextEdit::singleline(&mut app.set_weight_input)
                            .desired_width(60.0)
                            .hint_text("0"),
                    );
                    ui.label(RichText::new("kg").size(12.0).color(MUTED));
                    ui.label(RichText::new("回数:").size(12.0).color(MUTED));
                    ui.add(
                        egui::TextEdit::singleline(&mut app.set_reps_input)
                            .desired_width(40.0)
                            .hint_text("10"),
                    );
                    ui.label(RichText::new("回").size(12.0).color(MUTED));
                    if primary_button(ui, "+ セットを追加").clicked() {
                        app.add_set();
                    }
                });
            });
            ui.add_space(6.0);
        }

        // 今日の全セット（種目別グループ）
        if !app.session_sets.is_empty() {
            card(ui, |ui| {
                ui.label(RichText::new("今日の全セット").size(12.0).strong().color(TEXT_DARK));
                ui.add_space(4.0);

                // 種目でグルーピング
                let mut ex_ids: Vec<i64> = Vec::new();
                for s in &app.session_sets {
                    if !ex_ids.contains(&s.exercise_id) {
                        ex_ids.push(s.exercise_id);
                    }
                }

                for ex_id in ex_ids {
                    let group: Vec<&TrainingSet> = app.session_sets.iter()
                        .filter(|s| s.exercise_id == ex_id)
                        .collect();

                    if let Some(first) = group.first() {
                        let label = format!(
                            "▪ {} ({})",
                            first.exercise_name,
                            first.muscle_group
                        );
                        ui.label(RichText::new(label).size(12.0).strong().color(TEXT_DARK));
                        let detail: String = group.iter()
                            .map(|s| format!("Set{}: {}×{}", s.set_number, s.weight_kg, s.reps))
                            .collect::<Vec<_>>()
                            .join("  ");
                        ui.label(RichText::new(detail).size(11.0).color(MUTED));
                        ui.add_space(2.0);
                    }
                }
            });
        }

        ui.add_space(8.0);
    });
}

// ── 種目管理 ──────────────────────────────────────────────────────────────────

fn draw_exercises(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(2.0);

        // 新規追加 / 編集フォーム
        card(ui, |ui| {
            let is_editing = app.editing_exercise.is_some();
            let title = if is_editing { "種目を編集" } else { "新しい種目を登録" };
            ui.label(RichText::new(title).size(12.0).strong().color(TEXT_DARK));
            ui.add_space(4.0);

            let (name, muscle_group, notes) = if let Some((_, draft)) = &mut app.editing_exercise {
                (&mut draft.name, &mut draft.muscle_group, &mut draft.notes)
            } else {
                (
                    &mut app.new_exercise.name,
                    &mut app.new_exercise.muscle_group,
                    &mut app.new_exercise.notes,
                )
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new("名前:").size(12.0).color(MUTED));
                ui.add(
                    egui::TextEdit::singleline(name)
                        .desired_width(160.0)
                        .hint_text("ベンチプレス"),
                );
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("部位:").size(12.0).color(MUTED));
                egui::ComboBox::from_id_salt("muscle_group_combo")
                    .selected_text(if muscle_group.is_empty() { "未設定" } else { muscle_group.as_str() })
                    .show_ui(ui, |ui| {
                        for &grp in MUSCLE_GROUPS {
                            ui.selectable_value(muscle_group, grp.to_string(), if grp.is_empty() { "未設定" } else { grp });
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("メモ:").size(12.0).color(MUTED));
                ui.add(
                    egui::TextEdit::singleline(notes)
                        .desired_width(200.0)
                        .hint_text("任意"),
                );
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if is_editing {
                    if primary_button(ui, "保存").clicked() {
                        app.save_exercise_edit();
                    }
                    if ui.button(RichText::new("キャンセル").size(12.0).color(MUTED)).clicked() {
                        app.editing_exercise = None;
                    }
                } else {
                    if primary_button(ui, "+ 登録").clicked() {
                        app.add_exercise_master();
                    }
                }
            });
        });
        ui.add_space(6.0);

        // 一覧
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("種目一覧").size(12.0).strong().color(TEXT_DARK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.exercise_filter)
                            .desired_width(120.0)
                            .hint_text("検索..."),
                    );
                });
            });
            ui.add_space(4.0);

            if app.exercise_list.is_empty() {
                ui.label(RichText::new("種目が登録されていません").size(11.0).color(MUTED));
                return;
            }

            let query = app.exercise_filter.to_lowercase();
            let list: Vec<Exercise> = app.exercise_list.iter()
                .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query))
                .cloned()
                .collect();

            let mut edit_target: Option<(i64, ExerciseDraft)> = None;
            let mut delete_id: Option<i64> = None;

            for ex in &list {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&ex.name).size(12.0).color(TEXT_DARK));
                    if !ex.muscle_group.is_empty() {
                        ui.label(RichText::new(format!("[{}]", ex.muscle_group)).size(11.0).color(MUTED));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("削除").clicked() {
                            delete_id = Some(ex.id);
                        }
                        if ui.small_button("編集").clicked() {
                            edit_target = Some((ex.id, ExerciseDraft {
                                name:         ex.name.clone(),
                                muscle_group: ex.muscle_group.clone(),
                                notes:        ex.notes.clone(),
                            }));
                        }
                    });
                });
                if !ex.notes.is_empty() {
                    ui.label(RichText::new(format!("  {}", ex.notes)).size(10.0).color(MUTED));
                }
                ui.separator();
            }

            if let Some(target) = edit_target {
                app.editing_exercise = Some(target);
            }
            if let Some(id) = delete_id {
                app.delete_exercise_master(id);
            }
        });

        ui.add_space(8.0);
    });
}

// ── 履歴 ──────────────────────────────────────────────────────────────────────

fn draw_history(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(2.0);

        // 期間切り替え
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("表示期間:").size(12.0).color(MUTED));
                for &days in &[30u32, 60, 90] {
                    let is_active = app.training_history_days == days;
                    let btn = ui.add(
                        egui::Button::new(
                            RichText::new(format!("{}日", days))
                                .size(11.0)
                                .color(if is_active { Color32::WHITE } else { MUTED }),
                        )
                        .fill(if is_active { ACCENT } else { Color32::from_rgb(235, 225, 245) })
                        .rounding(egui::Rounding::same(12.0)),
                    );
                    if btn.clicked() && !is_active {
                        app.training_history_days = days;
                        app.refresh_training_history();
                    }
                }
            });
        });
        ui.add_space(6.0);

        if app.training_history.is_empty() {
            card(ui, |ui| {
                ui.label(RichText::new("トレーニング履歴がありません").size(12.0).color(MUTED));
            });
            return;
        }

        let history = app.training_history.clone();
        let mut toggle_id: Option<i64> = None;

        for (sess, sets) in &history {
            card(ui, |ui| {
                let is_expanded = app.training_history_expanded.contains(&sess.id);

                // 種目数とセット数を計算
                let ex_count = {
                    let mut ids: Vec<i64> = sets.iter().map(|s| s.exercise_id).collect();
                    ids.dedup();
                    ids.len()
                };
                let set_count = sets.len();

                let arrow = if is_expanded { "▼" } else { "▶" };
                let header = format!(
                    "{} {}  {} 種目 / {} セット{}",
                    arrow,
                    &sess.date,
                    ex_count,
                    set_count,
                    if sess.memo.is_empty() { String::new() } else { format!("  ({})", sess.memo) }
                );
                let header_resp = ui.add(
                    egui::Button::new(
                        RichText::new(header)
                            .size(12.0)
                            .color(if is_expanded { ACCENT } else { TEXT_DARK }),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
                );
                if header_resp.clicked() {
                    toggle_id = Some(sess.id);
                }

                if is_expanded && !sets.is_empty() {
                    ui.add_space(4.0);
                    // 種目でグルーピング
                    let mut ex_ids: Vec<i64> = Vec::new();
                    for s in sets {
                        if !ex_ids.contains(&s.exercise_id) {
                            ex_ids.push(s.exercise_id);
                        }
                    }
                    for ex_id in ex_ids {
                        let group: Vec<&TrainingSet> = sets.iter()
                            .filter(|s| s.exercise_id == ex_id)
                            .collect();
                        if let Some(first) = group.first() {
                            let label = format!("  {} ({})", first.exercise_name, first.muscle_group);
                            ui.label(RichText::new(label).size(11.0).strong().color(TEXT_DARK));
                            let detail: String = group.iter()
                                .map(|s| format!("Set{}: {}×{}", s.set_number, s.weight_kg, s.reps))
                                .collect::<Vec<_>>()
                                .join("  ");
                            ui.label(RichText::new(format!("    {}", detail)).size(11.0).color(MUTED));
                        }
                    }
                }
            });
            ui.add_space(4.0);
        }

        if let Some(id) = toggle_id {
            if app.training_history_expanded.contains(&id) {
                app.training_history_expanded.remove(&id);
            } else {
                app.training_history_expanded.insert(id);
            }
        }

        ui.add_space(8.0);
    });
}

// ── AI 分析 ───────────────────────────────────────────────────────────────────

fn draw_analysis(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(2.0);

        // 種目選択
        card(ui, |ui| {
            ui.label(RichText::new("種目を選ぶ").size(12.0).strong().color(TEXT_DARK));
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.analysis_filter)
                    .desired_width(ui.available_width())
                    .hint_text("種目を検索..."),
            );
            ui.add_space(4.0);

            let query = app.analysis_filter.to_lowercase();
            let exercises: Vec<Exercise> = app.exercise_list.iter()
                .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query))
                .cloned()
                .collect();

            if exercises.is_empty() {
                ui.label(RichText::new("種目がありません").size(11.0).color(MUTED));
            } else {
                let cols = 3usize;
                let rows = (exercises.len() + cols - 1) / cols;
                for row in 0..rows {
                    ui.horizontal(|ui| {
                        for col in 0..cols {
                            let idx = row * cols + col;
                            if idx >= exercises.len() { break; }
                            let ex = &exercises[idx];
                            let is_sel = app.analysis_exercise.as_ref().map(|e| e.id) == Some(ex.id);
                            let btn = ui.add(
                                egui::Button::new(
                                    RichText::new(&ex.name)
                                        .size(11.0)
                                        .color(if is_sel { Color32::WHITE } else { TEXT_DARK }),
                                )
                                .fill(if is_sel { ACCENT } else { Color32::from_rgb(240, 235, 250) })
                                .rounding(egui::Rounding::same(6.0))
                                .min_size(egui::vec2(80.0, 26.0)),
                            );
                            if btn.clicked() {
                                let ex_clone = ex.clone();
                                app.select_analysis_exercise(ex_clone);
                            }
                        }
                    });
                }
            }
        });
        ui.add_space(6.0);

        // 選択中の種目の進捗グラフ + 統計
        if let Some(ref ex) = app.analysis_exercise.clone() {
            let progress = app.exercise_progress.clone();

            card(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} ({})", ex.name, ex.muscle_group))
                        .size(13.0).strong().color(ACCENT),
                );
                ui.add_space(6.0);

                if progress.is_empty() {
                    ui.label(RichText::new("記録がありません（過去90日）").size(12.0).color(MUTED));
                } else {
                    // 統計
                    let max_w = progress.iter().map(|(_, w)| *w).fold(f64::NEG_INFINITY, f64::max);
                    let avg_w = progress.iter().map(|(_, w)| *w).sum::<f64>() / progress.len() as f64;
                    let sessions = progress.len();

                    ui.columns(3, |cols| {
                        cols[0].vertical_centered(|ui| {
                            ui.label(RichText::new("最大重量").size(11.0).color(MUTED));
                            ui.label(RichText::new(format!("{:.1}kg", max_w)).size(18.0).color(ACCENT).strong());
                        });
                        cols[1].vertical_centered(|ui| {
                            ui.label(RichText::new("平均MAX").size(11.0).color(MUTED));
                            ui.label(RichText::new(format!("{:.1}kg", avg_w)).size(18.0).color(ACCENT).strong());
                        });
                        cols[2].vertical_centered(|ui| {
                            ui.label(RichText::new("記録回数").size(11.0).color(MUTED));
                            ui.label(RichText::new(format!("{}回", sessions)).size(18.0).color(ACCENT).strong());
                        });
                    });

                    ui.add_space(8.0);
                    draw_progress_graph(ui, &progress);
                }
            });
        }

        ui.add_space(8.0);
    });
}

// ── 進捗グラフ（最大重量推移）────────────────────────────────────────────────

fn draw_progress_graph(ui: &mut Ui, data: &[(String, f64)]) {
    let desired = egui::vec2(ui.available_width(), 180.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, Color32::from_rgb(248, 242, 255));

    if data.is_empty() {
        return;
    }

    let n = data.len();
    let pad_x = 14.0f32;
    let pad_y = 20.0f32;
    let plot_w = rect.width()  - pad_x * 2.0;
    let plot_h = rect.height() - pad_y * 2.0;

    let min_w = data.iter().map(|(_, w)| *w).fold(f64::INFINITY,     f64::min);
    let max_w = data.iter().map(|(_, w)| *w).fold(f64::NEG_INFINITY, f64::max);
    let range  = (max_w - min_w).max(1.0);

    let to_pos = |i: usize, w: f64| -> Pos2 {
        let x = rect.left() + pad_x + (i as f32 / (n - 1).max(1) as f32) * plot_w;
        let y = rect.bottom() - pad_y - ((w - min_w) / range) as f32 * plot_h;
        Pos2::new(x, y)
    };

    // グリッド線
    for step in 0..=4 {
        let ratio = step as f32 / 4.0;
        let y = rect.bottom() - pad_y - ratio * plot_h;
        let w = min_w + ratio as f64 * range;
        painter.line_segment(
            [Pos2::new(rect.left() + pad_x, y), Pos2::new(rect.right() - pad_x, y)],
            Stroke::new(0.5, Color32::from_rgb(210, 195, 230)),
        );
        painter.text(
            Pos2::new(rect.left() + pad_x - 2.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("{:.0}", w),
            egui::FontId::proportional(9.0),
            MUTED,
        );
    }

    // 折れ線
    for i in 1..n {
        let p0 = to_pos(i - 1, data[i - 1].1);
        let p1 = to_pos(i,     data[i].1);
        painter.line_segment([p0, p1], Stroke::new(2.0, ACCENT));
        let b0 = Pos2::new(p0.x, rect.bottom() - pad_y);
        let b1 = Pos2::new(p1.x, rect.bottom() - pad_y);
        painter.add(egui::epaint::PathShape {
            points: vec![p0, p1, b1, b0],
            closed: true,
            fill: Color32::from_rgba_premultiplied(232, 121, 160, 25),
            stroke: egui::epaint::PathStroke::NONE,
        });
    }

    for i in 0..n {
        painter.circle_filled(to_pos(i, data[i].1), 3.0, ACCENT);
    }

    // X 軸ラベル
    let step = (n / 5).max(1);
    let mut label_indices: Vec<usize> = (0..n).step_by(step).collect();
    if *label_indices.last().unwrap() != n - 1 {
        label_indices.push(n - 1);
    }
    for i in label_indices {
        let p = to_pos(i, data[i].1);
        let date_str = if data[i].0.len() >= 10 { &data[i].0[5..10] } else { &data[i].0 };
        painter.text(
            Pos2::new(p.x, rect.bottom() - 2.0),
            egui::Align2::CENTER_BOTTOM,
            date_str,
            egui::FontId::proportional(9.0),
            MUTED,
        );
    }
}
