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
            .with_title("Diet Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "Diet Tracker",
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

/// ライト＋パステルテーマ。クリーム背景にコーラルピンクのアクセント。
fn setup_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();

    // ── 背景色の階層 ─────────────────────────────────────────────────────
    let bg_deep   = egui::Color32::from_rgb(254, 249, 255); // #fef9ff クリーム
    let bg_panel  = egui::Color32::from_rgb(248, 242, 252); // #f8f2fc 薄ラベンダー
    let bg_widget = egui::Color32::from_rgb(237, 229, 248); // #ede5f8 淡紫

    // ── テキスト色 ────────────────────────────────────────────────────────
    let text_primary = egui::Color32::from_rgb(60, 40, 70);   // 深紫
    let text_muted   = egui::Color32::from_rgb(160, 140, 175); // くすみ紫

    // ── アクセント色（コーラルピンク） ────────────────────────────────────
    let accent = egui::Color32::from_rgb(232, 121, 160); // #e879a0

    visuals.panel_fill       = bg_panel;
    visuals.window_fill      = bg_panel;
    visuals.extreme_bg_color = bg_deep;
    visuals.faint_bg_color   = bg_widget;

    visuals.widgets.noninteractive.bg_fill   = bg_widget;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(0.5, text_muted);
    visuals.widgets.inactive.bg_fill         = bg_widget;
    visuals.widgets.inactive.fg_stroke       = egui::Stroke::new(0.5, text_primary);
    visuals.widgets.hovered.bg_fill          = egui::Color32::from_rgb(255, 220, 235);
    visuals.widgets.hovered.fg_stroke        = egui::Stroke::new(1.0, accent);
    visuals.widgets.active.bg_fill           = accent;
    visuals.widgets.active.fg_stroke         = egui::Stroke::new(1.0, egui::Color32::WHITE);

    visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(232, 121, 160, 50);
    visuals.selection.stroke  = egui::Stroke::new(1.0, accent);

    // 全ウィジェットに角丸
    let rounding = egui::Rounding::same(8.0);
    visuals.window_rounding                 = egui::Rounding::same(12.0);
    visuals.widgets.noninteractive.rounding = rounding;
    visuals.widgets.inactive.rounding       = rounding;
    visuals.widgets.hovered.rounding        = rounding;
    visuals.widgets.active.rounding         = rounding;

    // 枠線を薄く
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(0.5, egui::Color32::from_rgb(220, 205, 235));
    visuals.widgets.inactive.bg_stroke       = egui::Stroke::new(0.5, egui::Color32::from_rgb(220, 205, 235));

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing   = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.window_margin  = egui::Margin::same(14.0);
    ctx.set_style(style);
}
