//! レポートスクリーン
//!
//! # レイアウト（上から）
//! 1. 日曜日通知（日曜のみ）
//! 2. 週次食事集計テーブル
//! 3. AI コーチカード（生成 ＋ 現在のレポート）
//! 4. 過去のレポート履歴（折りたたみリスト）

use egui::{Color32, RichText, ScrollArea, Ui};

use crate::app::App;
use crate::ui::{card, ACCENT, C_COLOR, F_COLOR, KCAL_COLOR, MUTED, P_COLOR, TEXT_DARK};

pub fn draw(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        if app.is_sunday {
            draw_sunday_notice(ui);
            ui.add_space(6.0);
        }

        draw_weekly_table(app, ui);
        ui.add_space(8.0);

        draw_ai_coach_card(app, ui);
        ui.add_space(8.0);

        draw_history_section(app, ui);
        ui.add_space(8.0);
    });
}

fn draw_sunday_notice(ui: &mut Ui) {
    egui::Frame::none()
        .fill(Color32::from_rgb(255, 240, 248))
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(12.0))
        .stroke(egui::Stroke::new(0.8, Color32::from_rgb(240, 190, 215)))
        .show(ui, |ui| {
            ui.label(
                RichText::new("🌸 今日は日曜日です。今週の振り返りレポートを生成しましょう！")
                    .size(12.0)
                    .color(ACCENT),
            );
        });
}

fn draw_weekly_table(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.label(RichText::new("直近 7 日間の食事記録").size(13.0).strong().color(TEXT_DARK));
        ui.add_space(4.0);

        if app.weekly_days.is_empty() {
            ui.label(RichText::new("記録がありません").color(MUTED).size(12.0));
            return;
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new("日付").size(11.0).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new("C(g) ").size(11.0).color(C_COLOR));
                ui.label(RichText::new("F(g) ").size(11.0).color(F_COLOR));
                ui.label(RichText::new("P(g) ").size(11.0).color(P_COLOR));
                ui.label(RichText::new("kcal  ").size(11.0).color(KCAL_COLOR));
            });
        });
        ui.separator();

        let target    = app.target_kcal;
        let days      = app.weekly_days.clone();
        let mut total = 0.0f64;

        for day in &days {
            total += day.kcal;
            let row_color = if day.kcal > target * 1.1 {
                Color32::from_rgb(220, 80, 60)
            } else {
                TEXT_DARK
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(date_short(&day.date)).size(12.0).color(row_color));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{:>5.0}", day.c)).size(11.0).color(C_COLOR));
                    ui.label(RichText::new(format!("{:>5.0}", day.f)).size(11.0).color(F_COLOR));
                    ui.label(RichText::new(format!("{:>5.0}", day.p)).size(11.0).color(P_COLOR));
                    ui.label(RichText::new(format!("{:>6.0}", day.kcal)).size(11.0).color(KCAL_COLOR));
                });
            });
        }

        ui.separator();
        let n   = days.len() as f64;
        let avg = if n > 0.0 { total / n } else { 0.0 };
        let diff = avg - target;
        let avg_color = match diff {
            d if d >  100.0 => Color32::from_rgb(220, 80, 60),
            d if d < -100.0 => Color32::from_rgb(60, 180, 100),
            _                => KCAL_COLOR,
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new("平均").size(11.0).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{:.0} kcal  (目標 {:.0} / 差 {:+.0})", avg, target, diff))
                        .size(11.0)
                        .color(avg_color),
                );
            });
        });
    });
}

fn draw_ai_coach_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("✨ AI コーチの評価").size(13.0).strong().color(TEXT_DARK));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn_label = if app.report_loading { "生成中..." } else { "レポートを生成" };
                if ui
                    .add_enabled(
                        !app.report_loading,
                        egui::Button::new(RichText::new(btn_label).size(12.0).color(Color32::WHITE))
                            .fill(ACCENT)
                            .rounding(egui::Rounding::same(8.0)),
                    )
                    .clicked()
                {
                    app.weekly_days = app.db.get_weekly_summaries().unwrap_or_default();
                    app.start_report();
                }
            });
        });

        if app.report_loading {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(RichText::new("Gemini が評価中です...").color(MUTED).size(12.0));
            });
        }

        if let Some(text) = &app.report_text {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(255, 248, 252))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(text).size(12.0).color(TEXT_DARK));
                });
        }

        if std::env::var("GEMINI_API_KEY")
            .map(|k| k.trim().is_empty() || k == "your_gemini_api_key_here")
            .unwrap_or(true)
        {
            ui.add_space(6.0);
            ui.label(
                RichText::new("⚠ GEMINI_API_KEY が未設定です。.env ファイルに設定してください。")
                    .size(11.0)
                    .color(KCAL_COLOR),
            );
        }
    });
}

fn draw_history_section(app: &mut App, ui: &mut Ui) {
    if app.report_history.is_empty() {
        return;
    }

    card(ui, |ui| {
        ui.label(RichText::new("📋 過去のレポート").size(13.0).strong().color(TEXT_DARK));
        ui.add_space(4.0);

        let history = app.report_history.clone();
        let mut toggle_id: Option<i64> = None;

        for report in &history {
            let is_expanded = app.history_expanded.contains(&report.id);

            // ヘッダー行（クリックで展開/折りたたみ）
            let arrow = if is_expanded { "▼" } else { "▶" };
            let header_resp = ui.add(
                egui::Button::new(
                    RichText::new(format!("{} {}", arrow, report.created_at))
                        .size(12.0)
                        .color(if is_expanded { ACCENT } else { TEXT_DARK }),
                )
                .fill(if is_expanded {
                    Color32::from_rgb(255, 240, 250)
                } else {
                    Color32::TRANSPARENT
                })
                .stroke(egui::Stroke::NONE),
            );

            if header_resp.clicked() {
                toggle_id = Some(report.id);
            }

            if is_expanded {
                egui::Frame::none()
                    .fill(Color32::from_rgb(255, 248, 252))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&report.text).size(12.0).color(TEXT_DARK));
                    });
                ui.add_space(4.0);
            }
            ui.separator();
        }

        // 展開状態のトグル（ループ外で実行）
        if let Some(id) = toggle_id {
            if app.history_expanded.contains(&id) {
                app.history_expanded.remove(&id);
            } else {
                app.history_expanded.insert(id);
            }
        }
    });
}

fn date_short(date: &str) -> String {
    if date.len() >= 10 { date[5..10].replace('-', "/") } else { date.to_owned() }
}
