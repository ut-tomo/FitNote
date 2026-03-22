//! エントリポイント

mod app;
mod db;
mod domain;
mod llm;
mod ui;

use app::App;
use db::Db;

fn main() -> eframe::Result {
    dotenvy::dotenv().ok();

    let db = Db::open("app.db").expect("データベースを開けませんでした");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 720.0])
            .with_min_inner_size([620.0, 580.0])
            .with_title("FitNote"),
        ..Default::default()
    };

    eframe::run_native(
        "FitNote",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(App::new(db)))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let japanese_font_candidates = [
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/Library/Fonts/Arial Unicode MS.ttf",
    ];

    let mut fonts = egui::FontDefinitions::default();

    for path in &japanese_font_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(data));
            for family in fonts.families.values_mut() {
                family.push("cjk".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

/// ライトなグレースケールテーマ。
fn setup_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    let bg_deep = egui::Color32::from_rgb(238, 239, 241);
    let bg_panel = egui::Color32::from_rgb(246, 247, 248);
    let bg_widget = egui::Color32::from_rgb(232, 234, 236);
    let text_primary = egui::Color32::from_rgb(34, 36, 38);
    let text_muted = egui::Color32::from_rgb(112, 118, 124);
    let accent = egui::Color32::from_rgb(72, 78, 84);
    let border = egui::Color32::from_rgb(205, 209, 214);
    let hover = egui::Color32::from_rgb(222, 225, 228);

    visuals.panel_fill       = bg_panel;
    visuals.window_fill      = bg_panel;
    visuals.extreme_bg_color = bg_deep;
    visuals.faint_bg_color   = bg_widget;

    visuals.widgets.noninteractive.bg_fill   = bg_widget;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(0.5, text_muted);
    visuals.widgets.inactive.bg_fill         = bg_widget;
    visuals.widgets.inactive.fg_stroke       = egui::Stroke::new(0.5, text_primary);
    visuals.widgets.hovered.bg_fill          = hover;
    visuals.widgets.hovered.fg_stroke        = egui::Stroke::new(1.0, accent);
    visuals.widgets.active.bg_fill           = accent;
    visuals.widgets.active.fg_stroke         = egui::Stroke::new(1.0, egui::Color32::WHITE);

    visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(72, 78, 84, 40);
    visuals.selection.stroke  = egui::Stroke::new(1.0, accent);

    // 全ウィジェットに角丸
    let rounding = egui::Rounding::same(8.0);
    visuals.window_rounding                 = egui::Rounding::same(12.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding       = rounding;
    visuals.widgets.hovered.rounding        = rounding;
    visuals.widgets.active.rounding         = rounding;

    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.8, border);
    visuals.widgets.inactive.bg_stroke       = egui::Stroke::new(0.8, border);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin  = egui::Margin::same(14.0);
    ctx.set_style(style);
}
