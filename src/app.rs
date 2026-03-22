//! アプリケーション状態とビジネスロジック

use std::collections::HashSet;
use std::sync::mpsc;

use chrono::{Datelike, Local};
use egui::Context;

use crate::db::Db;
use crate::domain::{DaySummary, Exercise, ExerciseDraft, FoodDraft, FoodItem, LoggedItem, MealTemplate, MealTemplateDraftItem, ReportHistory, Slot, Summary, TrainingSession, TrainingSet};

// ── 画面識別子 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Meals,
    Graph,
    Training,
    Report,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MealSubScreen {
    Today,
    Foods,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingSubScreen {
    Today,
    Exercises,
    History,
    Analysis,
}

// ── アプリ状態 ────────────────────────────────────────────────────────────────

pub struct App {
    pub screen: Screen,
    pub meal_sub: MealSubScreen,
    pub today: String,
    pub is_sunday: bool,

    // ── 今日スクリーン ──────────────────────────────────────────────────────
    pub weight_input: String,
    pub active_slot: Slot,
    pub food_search: String,
    pub amount_input: String,
    pub selected_food: Option<FoodItem>,
    pub today_items: Vec<(Slot, LoggedItem)>,
    pub cached_summary: Summary,

    // ── 食品管理スクリーン ──────────────────────────────────────────────────
    pub food_list: Vec<FoodItem>,
    pub food_filter: String,
    pub new_food: FoodDraft,
    pub editing_food: Option<(i64, FoodDraft)>,
    pub meal_templates: Vec<MealTemplate>,
    pub new_meal_template_name: String,
    pub new_meal_template_items: Vec<MealTemplateDraftItem>,

    // ── グラフスクリーン ────────────────────────────────────────────────────
    pub weight_history: Vec<(String, f64)>,
    pub graph_range: u32,

    // ── レポートスクリーン ──────────────────────────────────────────────────
    pub weekly_days: Vec<DaySummary>,
    pub report_text: Option<String>,
    pub report_loading: bool,
    pub report_rx: Option<mpsc::Receiver<anyhow::Result<String>>>,
    pub report_history: Vec<ReportHistory>,
    /// 展開中のレポート履歴 ID セット
    pub history_expanded: HashSet<i64>,

    // ── 設定（Settings タブ）──────────────────────────────────────────────
    pub target_kcal: f64,
    pub target_kcal_input: String,
    pub target_weight: f64,
    pub target_weight_input: String,
    pub target_p: f64,
    pub target_p_input: String,
    pub target_f: f64,
    pub target_f_input: String,
    pub target_c: f64,
    pub target_c_input: String,
    /// "男性" / "女性" / "その他"
    pub user_gender: String,
    pub user_age_input: String,
    pub user_height_input: String,

    // ── 筋トレスクリーン ────────────────────────────────────────────────────
    pub training_sub: TrainingSubScreen,

    // 種目マスタ
    pub exercise_list: Vec<Exercise>,
    pub exercise_filter: String,
    pub new_exercise: ExerciseDraft,
    pub editing_exercise: Option<(i64, ExerciseDraft)>,

    // 今日のセッション
    pub training_session_id: Option<i64>,
    pub session_memo: String,
    pub session_sets: Vec<TrainingSet>,
    pub selected_exercise: Option<Exercise>,
    pub exercise_search: String,
    pub set_weight_input: String,
    pub set_reps_input: String,

    // 履歴
    pub training_history: Vec<(TrainingSession, Vec<TrainingSet>)>,
    pub training_history_days: u32,
    pub training_history_expanded: HashSet<i64>,

    // AI 分析
    pub analysis_exercise: Option<Exercise>,
    pub analysis_filter: String,
    pub exercise_progress: Vec<(String, f64)>,

    // ── ステータストースト ──────────────────────────────────────────────────
    pub status: Option<(String, f32)>,

    pub db: Db,
}

impl App {
    pub fn new(db: Db) -> Self {
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        let is_sunday = now.weekday() == chrono::Weekday::Sun;

        // 設定値をDBから読み込む
        let target_kcal: f64 = db.get_setting("target_kcal").and_then(|s| s.parse().ok()).unwrap_or(2000.0);
        let target_weight: f64 = db.get_setting("target_weight").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let target_p: f64 = db.get_setting("target_p").and_then(|s| s.parse().ok()).unwrap_or(150.0);
        let target_f: f64 = db.get_setting("target_f").and_then(|s| s.parse().ok()).unwrap_or(60.0);
        let target_c: f64 = db.get_setting("target_c").and_then(|s| s.parse().ok()).unwrap_or(250.0);
        let user_gender = db.get_setting("user_gender").unwrap_or_else(|| "男性".to_string());
        let user_age_input = db.get_setting("user_age").unwrap_or_default();
        let user_height_input = db.get_setting("user_height").unwrap_or_default();

        let weight_input = db.get_today_weight(&today).map(|w| format!("{:.1}", w)).unwrap_or_default();
        let cached_summary  = db.get_today_summary(&today).unwrap_or_default();
        let today_items     = db.get_today_items(&today).unwrap_or_default();
        let food_list       = db.list_foods().unwrap_or_default();
        let weight_history  = db.get_weight_history(30).unwrap_or_default();
        let weekly_days     = db.get_weekly_summaries().unwrap_or_default();
        let report_history  = db.get_report_history().unwrap_or_default();
        let exercise_list   = db.list_exercises().unwrap_or_default();
        let meal_templates  = db.list_meal_templates().unwrap_or_default();

        // 今日のトレーニングセッション（存在する場合のみ読み込む）
        let (training_session_id, session_memo, session_sets) =
            if let Ok(Some(sess)) = db.get_session_by_date(&today) {
                let sets = db.get_session_sets(sess.id).unwrap_or_default();
                let memo = sess.memo.clone();
                (Some(sess.id), memo, sets)
            } else {
                (None, String::new(), Vec::new())
            };

        let training_history = db.get_training_history(30).unwrap_or_default();

        let screen = if is_sunday { Screen::Report } else { Screen::Meals };

        App {
            screen,
            meal_sub: MealSubScreen::Today,
            today,
            is_sunday,
            weight_input,
            active_slot: Slot::Breakfast,
            food_search: String::new(),
            amount_input: "1".into(),
            selected_food: None,
            today_items,
            cached_summary,
            food_list,
            food_filter: String::new(),
            new_food: FoodDraft::default(),
            editing_food: None,
            meal_templates,
            new_meal_template_name: String::new(),
            new_meal_template_items: vec![MealTemplateDraftItem::default()],
            weight_history,
            graph_range: 30,
            weekly_days,
            report_text: None,
            report_loading: false,
            report_rx: None,
            report_history,
            history_expanded: HashSet::new(),
            target_kcal,
            target_kcal_input: target_kcal.to_string(),
            target_weight,
            target_weight_input: if target_weight > 0.0 { format!("{:.1}", target_weight) } else { String::new() },
            target_p,
            target_p_input: target_p.to_string(),
            target_f,
            target_f_input: target_f.to_string(),
            target_c,
            target_c_input: target_c.to_string(),
            user_gender,
            user_age_input,
            user_height_input,
            training_sub: TrainingSubScreen::Today,
            exercise_list,
            exercise_filter: String::new(),
            new_exercise: ExerciseDraft::default(),
            editing_exercise: None,
            training_session_id,
            session_memo,
            session_sets,
            selected_exercise: None,
            exercise_search: String::new(),
            set_weight_input: String::new(),
            set_reps_input: "10".into(),
            training_history,
            training_history_days: 30,
            training_history_expanded: HashSet::new(),
            analysis_exercise: None,
            analysis_filter: String::new(),
            exercise_progress: Vec::new(),
            status: None,
            db,
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // キャッシュ更新
    // ════════════════════════════════════════════════════════════════════════

    pub fn refresh_today(&mut self) {
        self.cached_summary = self.db.get_today_summary(&self.today).unwrap_or_default();
        self.today_items    = self.db.get_today_items(&self.today).unwrap_or_default();
    }

    pub fn refresh_foods(&mut self) {
        self.food_list = self.db.list_foods().unwrap_or_default();
    }

    pub fn refresh_meal_templates(&mut self) {
        self.meal_templates = self.db.list_meal_templates().unwrap_or_default();
    }

    pub fn refresh_weight(&mut self) {
        self.weight_history = self.db.get_weight_history(self.graph_range).unwrap_or_default();
    }

    pub fn refresh_report_history(&mut self) {
        self.report_history = self.db.get_report_history().unwrap_or_default();
    }

    // ════════════════════════════════════════════════════════════════════════
    // UI ヘルパー
    // ════════════════════════════════════════════════════════════════════════

    pub fn toast(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), 3.0));
    }

    pub fn filtered_foods(&self) -> Vec<&FoodItem> {
        self.filter_foods(&self.food_search)
    }

    pub fn filtered_foods_manage(&self) -> Vec<&FoodItem> {
        self.filter_foods(&self.food_filter)
    }

    fn filter_foods(&self, query: &str) -> Vec<&FoodItem> {
        let q = query.to_lowercase();
        self.food_list
            .iter()
            .filter(|f| q.is_empty() || f.name.to_lowercase().contains(&q))
            .collect()
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：体重
    // ════════════════════════════════════════════════════════════════════════

    pub fn save_weight(&mut self) {
        match self.weight_input.trim().parse::<f64>() {
            Ok(kg) if (0.0..500.0).contains(&kg) => {
                match self.db.upsert_weight(&self.today, kg) {
                    Ok(()) => {
                        self.refresh_weight();
                        self.toast(format!("体重を保存しました: {:.1} kg", kg));
                    }
                    Err(e) => self.toast(format!("エラー: {}", e)),
                }
            }
            _ => self.toast("有効な体重を入力してください"),
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：食事記録
    // ════════════════════════════════════════════════════════════════════════

    pub fn log_meal_item(&mut self) {
        let food = match self.selected_food.clone() {
            Some(f) => f,
            None => { self.toast("食品を選択してください"); return; }
        };
        let amount = match self.amount_input.trim().parse::<f64>() {
            Ok(a) if a > 0.0 => a,
            _ => { self.toast("有効な量を入力してください"); return; }
        };

        let slot  = self.active_slot;
        let today = self.today.clone();

        match self.db.get_or_create_meal_log(&today, slot) {
            Ok(meal_id) => match self.db.add_meal_item(meal_id, food.id, amount) {
                Ok(()) => {
                    self.refresh_today();
                    self.food_search.clear();
                    self.selected_food = None;
                    self.amount_input = "1".into();
                    self.toast(format!("「{}」を追加しました", food.name));
                }
                Err(e) => self.toast(format!("エラー: {}", e)),
            },
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn delete_meal_item(&mut self, id: i64) {
        match self.db.delete_meal_item(id) {
            Ok(()) => { self.refresh_today(); self.toast("削除しました"); }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn add_meal_template_row(&mut self) {
        self.new_meal_template_items.push(MealTemplateDraftItem::default());
    }

    pub fn save_meal_template(&mut self) {
        let name = self.new_meal_template_name.trim().to_string();
        if name.is_empty() {
            self.toast("ショートカット名を入力してください");
            return;
        }

        let mut items: Vec<(i64, f64)> = Vec::new();
        for row in &self.new_meal_template_items {
            let food_id = match row.food_id {
                Some(id) => id,
                None => continue,
            };
            let amount = match row.amount.trim().parse::<f64>() {
                Ok(v) if v > 0.0 => v,
                _ => {
                    self.toast("ショートカット内の量は 0 より大きくしてください");
                    return;
                }
            };
            items.push((food_id, amount));
        }

        if items.is_empty() {
            self.toast("ショートカットに食材を 1 つ以上入れてください");
            return;
        }

        match self.db.add_meal_template(&name, &items) {
            Ok(()) => {
                self.new_meal_template_name.clear();
                self.new_meal_template_items = vec![MealTemplateDraftItem::default()];
                self.refresh_meal_templates();
                self.toast(format!("ショートカット「{}」を登録しました", name));
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn apply_meal_template(&mut self, template_id: i64) {
        let Some(template) = self.meal_templates.iter().find(|t| t.id == template_id).cloned() else {
            self.toast("ショートカットが見つかりません");
            return;
        };

        let slot = self.active_slot;
        let today = self.today.clone();
        match self.db.get_or_create_meal_log(&today, slot) {
            Ok(meal_id) => match self.db.add_meal_template_to_log(meal_id, template_id) {
                Ok(()) => {
                    self.refresh_today();
                    self.toast(format!("「{}」を追加しました", template.name));
                }
                Err(e) => self.toast(format!("エラー: {}", e)),
            },
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn delete_meal_template(&mut self, id: i64) {
        match self.db.delete_meal_template(id) {
            Ok(()) => {
                self.refresh_meal_templates();
                self.toast("ショートカットを削除しました");
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：食品マスタ
    // ════════════════════════════════════════════════════════════════════════

    pub fn add_food(&mut self) {
        if self.new_food.name.trim().is_empty() {
            self.toast("食品名を入力してください");
            return;
        }
        let draft = self.new_food.clone();
        match self.db.add_food(&draft) {
            Ok(()) => {
                self.toast(format!("「{}」を登録しました", draft.name));
                self.new_food = FoodDraft::default();
                self.refresh_foods();
                self.refresh_meal_templates();
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn save_food_edit(&mut self) {
        if let Some((id, draft)) = self.editing_food.take() {
            match self.db.update_food(id, &draft) {
                Ok(()) => {
                    self.toast("更新しました");
                    self.refresh_foods();
                    self.refresh_meal_templates();
                }
                Err(e) => {
                    self.editing_food = Some((id, draft));
                    self.toast(format!("エラー: {}", e));
            }
            }
        }
    }

    pub fn delete_food(&mut self, id: i64) {
        match self.db.delete_food(id) {
            Ok(()) => {
                self.toast("食品を削除しました");
                self.refresh_foods();
                self.refresh_meal_templates();
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：設定
    // ════════════════════════════════════════════════════════════════════════

    pub fn save_settings(&mut self) {
        if let Ok(v) = self.target_kcal_input.trim().parse::<f64>() { self.target_kcal = v; }
        if let Ok(v) = self.target_weight_input.trim().parse::<f64>() { self.target_weight = v; }
        if let Ok(v) = self.target_p_input.trim().parse::<f64>() { self.target_p = v; }
        if let Ok(v) = self.target_f_input.trim().parse::<f64>() { self.target_f = v; }
        if let Ok(v) = self.target_c_input.trim().parse::<f64>() { self.target_c = v; }

        let _ = self.db.set_setting("target_kcal",    &self.target_kcal.to_string());
        let _ = self.db.set_setting("target_weight",  &self.target_weight.to_string());
        let _ = self.db.set_setting("target_p",       &self.target_p.to_string());
        let _ = self.db.set_setting("target_f",       &self.target_f.to_string());
        let _ = self.db.set_setting("target_c",       &self.target_c.to_string());
        let _ = self.db.set_setting("user_gender",    &self.user_gender.clone());
        let _ = self.db.set_setting("user_age",       &self.user_age_input.clone());
        let _ = self.db.set_setting("user_height",    &self.user_height_input.clone());

        self.toast("設定を保存しました");
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：筋トレ
    // ════════════════════════════════════════════════════════════════════════

    pub fn refresh_training_today(&mut self) {
        if let Some(session_id) = self.training_session_id {
            self.session_sets = self.db.get_session_sets(session_id).unwrap_or_default();
        } else {
            self.session_sets.clear();
        }
    }

    pub fn refresh_training_history(&mut self) {
        self.training_history = self.db
            .get_training_history(self.training_history_days)
            .unwrap_or_default();
    }

    /// セットを追加する。セッションがなければ作成する。
    pub fn add_set(&mut self) {
        let exercise = match self.selected_exercise.clone() {
            Some(e) => e,
            None => { self.toast("種目を選択してください"); return; }
        };
        let weight = match self.set_weight_input.trim().parse::<f64>() {
            Ok(w) if w >= 0.0 => w,
            _ => { self.toast("有効な重量を入力してください"); return; }
        };
        let reps = match self.set_reps_input.trim().parse::<i32>() {
            Ok(r) if r > 0 => r,
            _ => { self.toast("有効な回数を入力してください"); return; }
        };

        let today = self.today.clone();
        let session_id = match self.training_session_id {
            Some(id) => id,
            None => match self.db.get_or_create_today_session(&today) {
                Ok(id) => { self.training_session_id = Some(id); id }
                Err(e) => { self.toast(format!("エラー: {}", e)); return; }
            }
        };

        // 今の種目の最大 set_number を取得
        let current_max = self.session_sets.iter()
            .filter(|s| s.exercise_id == exercise.id)
            .map(|s| s.set_number)
            .max()
            .unwrap_or(0);

        match self.db.add_training_set(session_id, exercise.id, current_max + 1, reps, weight) {
            Ok(()) => {
                self.refresh_training_today();
                self.toast(format!("Set {} 追加: {}kg × {}reps", current_max + 1, weight, reps));
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn delete_set(&mut self, id: i64) {
        match self.db.delete_training_set(id) {
            Ok(()) => { self.refresh_training_today(); self.toast("削除しました"); }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn add_exercise_master(&mut self) {
        if self.new_exercise.name.trim().is_empty() {
            self.toast("種目名を入力してください");
            return;
        }
        let draft = self.new_exercise.clone();
        match self.db.add_exercise(&draft) {
            Ok(()) => {
                self.toast(format!("「{}」を登録しました", draft.name));
                self.new_exercise = ExerciseDraft::default();
                self.exercise_list = self.db.list_exercises().unwrap_or_default();
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    pub fn save_exercise_edit(&mut self) {
        if let Some((id, draft)) = self.editing_exercise.take() {
            match self.db.update_exercise(id, &draft) {
                Ok(()) => {
                    self.toast("更新しました");
                    self.exercise_list = self.db.list_exercises().unwrap_or_default();
                }
                Err(e) => {
                    self.editing_exercise = Some((id, draft));
                    self.toast(format!("エラー: {}", e));
                }
            }
        }
    }

    pub fn delete_exercise_master(&mut self, id: i64) {
        match self.db.delete_exercise(id) {
            Ok(()) => {
                self.toast("種目を削除しました");
                self.exercise_list = self.db.list_exercises().unwrap_or_default();
            }
            Err(e) => self.toast(format!("エラー: {}", e)),
        }
    }

    /// 種目を選択して最大重量推移を取得する。
    pub fn select_analysis_exercise(&mut self, ex: Exercise) {
        self.exercise_progress = self.db
            .get_exercise_max_weight_history(ex.id, 90)
            .unwrap_or_default();
        self.analysis_exercise = Some(ex);
    }

    // ════════════════════════════════════════════════════════════════════════
    // ビジネスロジック：LLM レポート
    // ════════════════════════════════════════════════════════════════════════

    pub fn start_report(&mut self) {
        if std::env::var("GEMINI_API_KEY")
            .map(|k| k.trim().is_empty() || k == "your_gemini_api_key_here")
            .unwrap_or(true)
        {
            self.toast("GEMINI_API_KEY が .env に設定されていません");
            return;
        }

        let target_kcal = self.target_kcal;
        let weekly_days = self.weekly_days.clone();
        let weight_history = self.weight_history.clone();
        let training_summary = self.db.get_weekly_training_summary().unwrap_or_default();
        let (tx, rx) = mpsc::channel();
        self.report_rx      = Some(rx);
        self.report_loading = true;
        self.report_text    = None;

        std::thread::spawn(move || {
            let memo_entries: Vec<(String, String)> = training_summary
                .iter()
                .filter_map(|(date, _, _, memo)| {
                    let memo = memo.trim();
                    if memo.is_empty() {
                        None
                    } else {
                        Some((date.clone(), memo.to_string()))
                    }
                })
                .collect();

            let compressed_map = crate::llm::summarize_training_memos(&memo_entries)
                .unwrap_or_default()
                .into_iter()
                .collect::<std::collections::HashMap<_, _>>();

            let compressed_training_summary = training_summary
                .into_iter()
                .map(|(date, sets, volume, memo)| {
                    let compressed = compressed_map.get(&date).cloned().unwrap_or(memo);
                    (date, sets, volume, compressed)
                })
                .collect::<Vec<_>>();

            let prompt = App::build_llm_prompt(
                target_kcal,
                &weekly_days,
                &weight_history,
                &compressed_training_summary,
            );

            let _ = tx.send(crate::llm::generate_weekly_report(&prompt));
        });
    }

    fn build_llm_prompt(
        target_kcal: f64,
        weekly_days: &[DaySummary],
        weight_history: &[(String, f64)],
        training_summary: &[(String, i32, f64, String)],
    ) -> String {
        let mut prompt = format!(
            "あなたはダイエット＆筋トレコーチです。以下の直近7日間の食事・体重・トレーニングデータを分析し、\
             日本語で総評・良かった点・改善点・来週へのアドバイスを400〜500字でまとめてください。\n\n\
             目標摂取カロリー: {:.0} kcal/日\n\n食事記録:\n",
            target_kcal
        );

        for day in weekly_days {
            prompt.push_str(&format!(
                "  {}: {:.0} kcal (P {:.0}g / F {:.0}g / C {:.0}g)\n",
                day.date, day.kcal, day.p, day.f, day.c
            ));
        }

        if !weight_history.is_empty() {
            prompt.push_str("\n体重記録 (直近7日):\n");
            for (date, kg) in weight_history.iter().rev().take(7).rev() {
                prompt.push_str(&format!("  {}: {:.1} kg\n", date, kg));
            }
        }

        if !training_summary.is_empty() {
            prompt.push_str("\nトレーニング記録 (直近7日):\n");
            for (date, sets, volume, memo) in training_summary {
                if memo.trim().is_empty() {
                    prompt.push_str(&format!(
                        "  {}: {}セット, 総ボリューム {:.0}kg\n",
                        date, sets, volume
                    ));
                } else {
                    prompt.push_str(&format!(
                        "  {}: {}セット, 総ボリューム {:.0}kg, メモ: {}\n",
                        date, sets, volume, memo.trim()
                    ));
                }
            }
        }

        prompt
    }

    pub fn poll_report(&mut self) {
        if let Some(rx) = &self.report_rx {
            if let Ok(result) = rx.try_recv() {
                self.report_loading = false;
                self.report_rx      = None;
                match result {
                    Ok(text) => {
                        // 履歴に保存
                        let _ = self.db.save_report(&text);
                        self.refresh_report_history();
                        self.report_text = Some(text);
                    }
                    Err(e) => self.toast(format!("LLM エラー: {}", e)),
                }
            }
        }
    }
}

// ── egui レンダリングループ ───────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.poll_report();

        if let Some((_, ref mut remaining)) = self.status {
            *remaining -= ctx.input(|i| i.stable_dt);
            if *remaining <= 0.0 {
                self.status = None;
            }
        }

        if self.report_loading {
            ctx.request_repaint();
        }

        crate::ui::draw(self, ctx);
    }
}
