//! ドメイン型定義
//!
//! UI 層・DB 層の両方から参照される純粋なデータ型を定義する。
//! ビジネスロジックや I/O は含まない。

// ── エラー型 ──────────────────────────────────────────────────────────────────

/// アプリ全体で使う Result 型エイリアス。
pub type Result<T> = anyhow::Result<T>;

// ── 栄養素サマリ ──────────────────────────────────────────────────────────────

/// 1 日分または任意期間の合計栄養素。
#[derive(Debug, Clone, Copy, Default)]
pub struct Summary {
    pub kcal: f64,
    pub p: f64, // タンパク質 (g)
    pub f: f64, // 脂質 (g)
    pub c: f64, // 炭水化物 (g)
}

// ── 食事スロット ──────────────────────────────────────────────────────────────

/// 1 日の食事を区分するスロット（朝食 / 昼食 / 夕食 / 間食）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl Slot {
    /// DB 保存用の ASCII 文字列。
    pub fn as_str(self) -> &'static str {
        match self {
            Slot::Breakfast => "breakfast",
            Slot::Lunch => "lunch",
            Slot::Dinner => "dinner",
            Slot::Snack => "snack",
        }
    }

    /// 画面表示用のラベル。
    pub fn label(self) -> &'static str {
        match self {
            Slot::Breakfast => "朝食",
            Slot::Lunch => "昼食",
            Slot::Dinner => "夕食",
            Slot::Snack => "間食",
        }
    }

    /// 全スロットの配列。タブ描画や集計ループで使う。
    pub fn all() -> [Slot; 4] {
        [Slot::Breakfast, Slot::Lunch, Slot::Dinner, Slot::Snack]
    }

    /// DB から読み出した文字列を Slot に変換する。未知の値は Snack 扱い。
    pub fn from_str(s: &str) -> Self {
        match s {
            "breakfast" => Slot::Breakfast,
            "lunch" => Slot::Lunch,
            "dinner" => Slot::Dinner,
            _ => Slot::Snack,
        }
    }
}

// ── 食品の計量単位 ────────────────────────────────────────────────────────────

/// ユーザーが食品を計量するときの単位。
/// DB には `as_str()` の値を TEXT で保存する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Unit {
    #[default]
    G,
    Ml,
    Ko,        // 個
    Hon,       // 本
    Hai,       // 杯
    Kire,      // 切れ
    Mai,       // 枚
    Fukuro,    // 袋
    Hitokuchi, // 口
    Tsubu,     // 粒
    Pak,       // パック
    Serving,   // 食分
}

impl Unit {
    /// DB / 表示用の文字列。
    pub fn as_str(self) -> &'static str {
        match self {
            Unit::G => "g",
            Unit::Ml => "ml",
            Unit::Ko => "個",
            Unit::Hon => "本",
            Unit::Hai => "杯",
            Unit::Kire => "切れ",
            Unit::Mai => "枚",
            Unit::Fukuro => "袋",
            Unit::Hitokuchi => "口",
            Unit::Tsubu => "粒",
            Unit::Pak => "パック",
            Unit::Serving => "食分",
        }
    }

    /// ComboBox 表示などで使う全バリアントの列挙。
    pub fn all() -> &'static [Unit] {
        &[
            Unit::G,
            Unit::Ml,
            Unit::Ko,
            Unit::Hon,
            Unit::Hai,
            Unit::Kire,
            Unit::Mai,
            Unit::Fukuro,
            Unit::Hitokuchi,
            Unit::Tsubu,
            Unit::Pak,
            Unit::Serving,
        ]
    }

    /// DB から読み出した文字列を Unit に変換する。未知の値は G 扱い。
    pub fn from_str(s: &str) -> Self {
        match s {
            "g" => Unit::G,
            "ml" => Unit::Ml,
            "個" => Unit::Ko,
            "本" => Unit::Hon,
            "杯" => Unit::Hai,
            "切れ" => Unit::Kire,
            "枚" => Unit::Mai,
            "袋" => Unit::Fukuro,
            "口" => Unit::Hitokuchi,
            "粒" => Unit::Tsubu,
            "パック" => Unit::Pak,
            "食分" => Unit::Serving,
            _ => Unit::G,
        }
    }
}

// ── 食品マスタ ────────────────────────────────────────────────────────────────

/// DB の `food_item` テーブルに対応する食品マスタレコード。
#[derive(Debug, Clone)]
pub struct FoodItem {
    pub id: i64,
    pub name: String,
    /// 1 単位あたりの計量単位
    pub unit: Unit,
    /// 1 単位あたりのカロリー
    pub kcal_per_unit: f64,
    pub p_per_unit: f64,
    pub f_per_unit: f64,
    pub c_per_unit: f64,
}

/// 追加フォーム・編集フォーム用の入力バッファ。
/// 数値フィールドはバリデーションのために String のまま保持する。
#[derive(Debug, Clone, Default)]
pub struct FoodDraft {
    pub name: String,
    pub unit: Unit,
    /// カロリー入力値（文字列。保存時に f64 へパース）
    pub kcal: String,
    pub p: String,
    pub f: String,
    pub c: String,
}

impl FoodDraft {
    /// 既存 FoodItem を編集フォームに読み込む。
    pub fn from_item(item: &FoodItem) -> Self {
        FoodDraft {
            name: item.name.clone(),
            unit: item.unit,
            kcal: item.kcal_per_unit.to_string(),
            p: item.p_per_unit.to_string(),
            f: item.f_per_unit.to_string(),
            c: item.c_per_unit.to_string(),
        }
    }

    // 各数値フィールドの安全なパース（パース失敗時は 0.0）
    pub fn kcal_f(&self) -> f64 { self.kcal.parse().unwrap_or(0.0) }
    pub fn p_f(&self)    -> f64 { self.p.parse().unwrap_or(0.0) }
    pub fn f_f(&self)    -> f64 { self.f.parse().unwrap_or(0.0) }
    pub fn c_f(&self)    -> f64 { self.c.parse().unwrap_or(0.0) }
}

/// 食事ショートカットを構成する 1 食材分。
#[derive(Debug, Clone)]
pub struct MealTemplateItem {
    pub food_name: String,
    pub unit: Unit,
    pub amount: f64,
}

/// 「おでん」など複数食材を束ねた食事ショートカット。
#[derive(Debug, Clone)]
pub struct MealTemplate {
    pub id: i64,
    pub name: String,
    pub items: Vec<MealTemplateItem>,
}

/// 食事ショートカット作成フォームの 1 行。
#[derive(Debug, Clone, Default)]
pub struct MealTemplateDraftItem {
    pub food_id: Option<i64>,
    pub amount: String,
}

// ── ログ済み食事アイテム ──────────────────────────────────────────────────────

/// meal_log_item + food_item を JOIN して取得した 1 食品エントリ。
/// 画面表示・削除操作に必要なフィールドをまとめる。
#[derive(Debug, Clone)]
pub struct LoggedItem {
    /// meal_log_item の主キー。削除時に使用。
    pub id: i64,
    pub food_name: String,
    pub unit: Unit,
    pub amount: f64,
    /// 摂取量に換算済みの栄養素
    pub kcal: f64,
    #[allow(dead_code)]
    pub p: f64,
    #[allow(dead_code)]
    pub f: f64,
    #[allow(dead_code)]
    pub c: f64,
}

// ── 週次集計 ──────────────────────────────────────────────────────────────────

/// 1 日分の食事合計（週次レポートで使用）。
#[derive(Debug, Clone, Default)]
pub struct DaySummary {
    pub date: String,
    pub kcal: f64,
    pub p: f64,
    pub f: f64,
    pub c: f64,
}

// ── LLM レポート履歴 ──────────────────────────────────────────────────────────

/// 過去に生成された AI レポートの 1 件分。
#[derive(Debug, Clone)]
pub struct ReportHistory {
    pub id: i64,
    /// 生成日時（"YYYY-MM-DD HH:MM" 形式）
    pub created_at: String,
    pub text: String,
}

// ── 筋トレ ────────────────────────────────────────────────────────────────────

/// 種目マスタ。
#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: i64,
    pub name: String,
    /// 筋肉グループ（"胸" / "背中" / "脚" / "肩" / "腕" / "腹" / ""）
    pub muscle_group: String,
    pub notes: String,
}

/// 種目マスタの入力フォーム用バッファ。
#[derive(Debug, Clone, Default)]
pub struct ExerciseDraft {
    pub name: String,
    pub muscle_group: String,
    pub notes: String,
}

/// トレーニングセッション（1 日 1 件以上可）。
#[derive(Debug, Clone)]
pub struct TrainingSession {
    pub id: i64,
    pub date: String,
    pub memo: String,
}

/// セット記録（セッション × 種目 × セット番号）。
#[derive(Debug, Clone)]
pub struct TrainingSet {
    pub id: i64,
    #[allow(dead_code)]
    pub session_id: i64,
    pub exercise_id: i64,
    /// JOIN で取得した種目名
    pub exercise_name: String,
    /// JOIN で取得した筋肉グループ
    pub muscle_group: String,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
}
