//! UI 層のルートモジュール

pub mod foods;
pub mod graph;
pub mod home;
pub mod report;
pub mod settings;
pub mod training;

use crate::app::{App, MealSubScreen, Screen};
use egui::{Color32, Context, RichText};

// ── グレースケールパレット ───────────────────────────────────────────────────

pub const KCAL_COLOR: Color32 = Color32::from_rgb(78, 84, 90);
pub const P_COLOR: Color32 = Color32::from_rgb(102, 108, 114);
pub const F_COLOR: Color32 = Color32::from_rgb(126, 131, 137);
pub const C_COLOR: Color32 = Color32::from_rgb(150, 155, 160);
pub const ACCENT: Color32 = Color32::from_rgb(68, 74, 80);
pub const MUTED: Color32 = Color32::from_rgb(112, 118, 124);
pub const CARD_BG: Color32 = Color32::from_rgb(252, 252, 252);
pub const TEXT_DARK: Color32 = Color32::from_rgb(30, 33, 36);

// ── ルート描画 ────────────────────────────────────────────────────────────────

pub fn draw(app: &mut App, ctx: &Context) {
    draw_top_bar(app, ctx);
    draw_tab_bar(app, ctx);

    egui::CentralPanel::default().show(ctx, |ui| {
        match app.screen {
            Screen::Meals    => draw_meal_screen(app, ui),
            Screen::Graph    => graph::draw(app, ui),
            Screen::Training => training::draw(app, ui),
            Screen::Report   => report::draw(app, ui),
            Screen::Settings => settings::draw(app, ui),
        }
    });
}

fn draw_top_bar(app: &App, ctx: &Context) {
    egui::TopBottomPanel::top("top_bar")
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(243, 244, 245))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("FitNote")
                        .size(18.0)
                        .color(ACCENT)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some((msg, _)) = &app.status {
                        ui.colored_label(Color32::from_rgb(84, 94, 102), msg);
                    }
                });
            });
        });
}

fn draw_tab_bar(app: &mut App, ctx: &Context) {
    egui::TopBottomPanel::top("tab_bar")
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgb(236, 238, 240))
                .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                tab_button(ui, app, Screen::Meals,    " 食事 ");
                tab_button(ui, app, Screen::Graph,    "グラフ");
                tab_button(ui, app, Screen::Training, "筋トレ");
                tab_button(ui, app, Screen::Report,   "レポート");
                tab_button(ui, app, Screen::Settings, " 設定 ");
            });
        });
}

fn tab_button(ui: &mut egui::Ui, app: &mut App, screen: Screen, label: &str) {
    let is_active = app.screen == screen;
    let text_color = if is_active { ACCENT } else { MUTED };

    let btn = ui.add(
        egui::Button::new(RichText::new(label).color(text_color).size(13.0))
            .fill(Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE),
    );

    if is_active {
        let r = btn.rect;
        ui.painter().line_segment(
            [egui::pos2(r.left(), r.bottom()), egui::pos2(r.right(), r.bottom())],
            egui::Stroke::new(2.5, ACCENT),
        );
    }

    if btn.clicked() {
        app.screen = screen;
    }
}

fn draw_meal_screen(app: &mut App, ui: &mut egui::Ui) {
    draw_meal_sub_tab_bar(app, ui);
    ui.add_space(4.0);

    match app.meal_sub {
        MealSubScreen::Today => home::draw(app, ui),
        MealSubScreen::Foods => foods::draw(app, ui),
    }
}

fn draw_meal_sub_tab_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        meal_sub_tab_btn(ui, app, MealSubScreen::Today, "今日");
        meal_sub_tab_btn(ui, app, MealSubScreen::Foods, "食材管理");
    });
    ui.separator();
}

fn meal_sub_tab_btn(ui: &mut egui::Ui, app: &mut App, screen: MealSubScreen, label: &str) {
    let is_active = app.meal_sub == screen;
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
        app.meal_sub = screen;
    }
}

// ── 共有ウィジェット ──────────────────────────────────────────────────────────

/// 栄養素のプログレスバー（ラベル | バー | 値）。
pub fn nutrient_bar(ui: &mut egui::Ui, label: &str, value: f64, max: f64, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(MUTED).size(12.0));

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width() - 65.0, 10.0),
            egui::Sense::hover(),
        );
        // バー背景
        ui.painter().rect_filled(rect, 5.0, Color32::from_rgb(226, 229, 232));
        // バー塗り
        if max > 0.0 {
            let fill_ratio = (value / max).min(1.0) as f32;
            let mut fill = rect;
            fill.set_right(rect.left() + rect.width() * fill_ratio);
            ui.painter().rect_filled(fill, 5.0, color);
        }

        ui.label(RichText::new(format!("{:.0}", value)).color(color).size(12.0));
    });
}

/// 白背景・角丸・薄影のカードコンテナ。
pub fn card(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(CARD_BG)
        .rounding(egui::Rounding::same(10.0))
        .inner_margin(egui::Margin::same(14.0))
        .stroke(egui::Stroke::new(0.8, Color32::from_rgb(214, 218, 222)))
        .show(ui, add_contents);
}

/// アクセント色のプライマリボタン。
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(12.0).color(Color32::WHITE))
            .fill(ACCENT)
            .rounding(egui::Rounding::same(8.0)),
    )
}
