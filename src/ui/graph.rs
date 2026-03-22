//! グラフスクリーン
//!
//! 体重の折れ線グラフと統計情報を表示する。
//! グラフは egui の `Painter` API を使って手描きする（外部依存なし）。
//!
//! # レイアウト（上から）
//! 1. 期間切り替えボタン（30日 / 60日 / 90日）＋ グラフカード
//! 2. 統計カード（最新 / 平均 / 最低 / 最高）

use egui::{Color32, Pos2, RichText, ScrollArea, Stroke, Ui};

use crate::app::App;
use crate::ui::{card, ACCENT, MUTED, TEXT_DARK};

/// グラフスクリーン全体を描画する。
pub fn draw(app: &mut App, ui: &mut Ui) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.0);

        draw_graph_card(app, ui);
        ui.add_space(8.0);

        if !app.weight_history.is_empty() {
            draw_stats_card(app, ui);
        }

        ui.add_space(8.0);
    });
}

// ── サブ描画関数 ──────────────────────────────────────────────────────────────

/// 期間切り替えボタン ＋ 体重折れ線グラフのカード。
fn draw_graph_card(app: &mut App, ui: &mut Ui) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("体重の推移").size(13.0).strong().color(TEXT_DARK));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for &days in &[30u32, 60, 90] {
                    if range_button(ui, app, days).clicked() {
                        app.graph_range = days;
                        app.refresh_weight();
                    }
                }
            });
        });
        ui.add_space(8.0);

        let history = app.weight_history.clone();
        draw_weight_graph(ui, &history);
    });
}

/// 統計情報（最新 / 平均 / 最低 / 最高）を 4 列で表示するカード。
fn draw_stats_card(app: &App, ui: &mut Ui) {
    card(ui, |ui| {
        let weights: Vec<f64> = app.weight_history.iter().map(|(_, w)| *w).collect();

        let latest = *weights.last().unwrap();
        let avg    = weights.iter().sum::<f64>() / weights.len() as f64;
        let min    = weights.iter().cloned().fold(f64::INFINITY, f64::min);
        let max    = weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        ui.columns(4, |cols| {
            stat_column(&mut cols[0], "最新", latest);
            stat_column(&mut cols[1], "平均", avg);
            stat_column(&mut cols[2], "最低", min);
            stat_column(&mut cols[3], "最高", max);
        });
    });
}

// ── ウィジェットヘルパー ──────────────────────────────────────────────────────

/// 期間切り替えボタン（アクティブなら ACCENT 色で強調）。
fn range_button(ui: &mut Ui, app: &App, days: u32) -> egui::Response {
    let is_active = app.graph_range == days;
    ui.add(
        egui::Button::new(
            RichText::new(format!("{}日", days))
                .size(11.0)
                .color(if is_active { Color32::WHITE } else { MUTED }),
        )
        .fill(if is_active { ACCENT } else { Color32::from_rgb(32, 36, 54) })
        .rounding(egui::Rounding::same(14.0)),
    )
}

/// 1 統計項目を縦中央寄せで描画する（ラベル ＋ 値 ＋ 単位）。
fn stat_column(ui: &mut egui::Ui, label: &str, value: f64) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(label).size(11.0).color(MUTED));
        ui.label(
            RichText::new(format!("{:.1}", value))
                .size(18.0)
                .color(ACCENT)
                .strong(),
        );
        ui.label(RichText::new("kg").size(10.0).color(MUTED));
    });
}

// ── グラフ描画 ────────────────────────────────────────────────────────────────

/// `data` の体重履歴を折れ線グラフとして描画する。
///
/// # 描画内容
/// - グラフ背景
/// - 水平グリッド線（4 本）＋ Y 軸ラベル
/// - 折れ線（ACCENT 色）＋ 半透明の塗りつぶし
/// - データポイント（丸点）
/// - 日付ラベル（X 軸、数点のみ表示）
fn draw_weight_graph(ui: &mut Ui, data: &[(String, f64)]) {
    let desired = egui::vec2(ui.available_width(), 200.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, Color32::from_rgb(248, 242, 255));

    if data.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "体重データがありません",
            egui::FontId::proportional(13.0),
            MUTED,
        );
        return;
    }

    let n = data.len();

    // グラフ描画領域（内側パディング）
    let pad_x = 14.0f32;
    let pad_y = 20.0f32;
    let plot_w = rect.width()  - pad_x * 2.0;
    let plot_h = rect.height() - pad_y * 2.0;

    // データ範囲（最低・最高の差が 1 未満でも最低 1 の幅を確保）
    let min_kg = data.iter().map(|(_, w)| *w).fold(f64::INFINITY,     f64::min);
    let max_kg = data.iter().map(|(_, w)| *w).fold(f64::NEG_INFINITY, f64::max);
    let range  = (max_kg - min_kg).max(1.0);

    // データ点のインデックスと kg 値から画面座標へ変換するクロージャ
    let to_pos = |i: usize, kg: f64| -> Pos2 {
        let x = rect.left() + pad_x + (i as f32 / (n - 1).max(1) as f32) * plot_w;
        let y = rect.bottom() - pad_y - ((kg - min_kg) / range) as f32 * plot_h;
        Pos2::new(x, y)
    };

    // グリッド線と Y 軸ラベルを描画
    for step in 0..=4 {
        let ratio = step as f32 / 4.0;
        let y  = rect.bottom() - pad_y - ratio * plot_h;
        let kg = min_kg + ratio as f64 * range;

        painter.line_segment(
            [
                Pos2::new(rect.left() + pad_x, y),
                Pos2::new(rect.right() - pad_x, y),
            ],
            Stroke::new(0.5, Color32::from_rgb(210, 195, 230)),
        );
        painter.text(
            Pos2::new(rect.left() + pad_x - 2.0, y),
            egui::Align2::RIGHT_CENTER,
            format!("{:.0}", kg),
            egui::FontId::proportional(9.0),
            MUTED,
        );
    }

    // 折れ線 ＋ 塗りつぶし
    let line_color = ACCENT;
    for i in 1..n {
        let p0 = to_pos(i - 1, data[i - 1].1);
        let p1 = to_pos(i,     data[i].1);

        painter.line_segment([p0, p1], Stroke::new(2.0, line_color));

        // ライン下の半透明塗りつぶし（グラデーション風）
        let b0 = Pos2::new(p0.x, rect.bottom() - pad_y);
        let b1 = Pos2::new(p1.x, rect.bottom() - pad_y);
        painter.add(egui::epaint::PathShape {
            points: vec![p0, p1, b1, b0],
            closed: true,
            fill: Color32::from_rgba_premultiplied(232, 121, 160, 25),
            stroke: egui::epaint::PathStroke::NONE,
        });
    }

    // データポイント
    for i in 0..n {
        painter.circle_filled(to_pos(i, data[i].1), 3.0, line_color);
    }

    // X 軸日付ラベル（全データを均等に最大 5 点分表示）
    let step = (n / 5).max(1);
    let mut label_indices: Vec<usize> = (0..n).step_by(step).collect();
    if *label_indices.last().unwrap() != n - 1 {
        label_indices.push(n - 1);
    }

    for i in label_indices {
        let p = to_pos(i, data[i].1);
        let date_str = date_short(&data[i].0);
        painter.text(
            Pos2::new(p.x, rect.bottom() - 2.0),
            egui::Align2::CENTER_BOTTOM,
            date_str,
            egui::FontId::proportional(9.0),
            MUTED,
        );
    }
}

/// `YYYY-MM-DD` 形式の日付から `MM-DD` だけを切り出す。
fn date_short(date: &str) -> &str {
    if date.len() >= 10 { &date[5..10] } else { date }
}
