//! データベースアクセス層
//!
//! SQLite（rusqlite）を薄くラップした `Db` 構造体を提供する。
//! ビジネスロジックはここに書かず、SQL の実行と型変換のみを担う。
//!
//! # テーブル構成
//! - `food_item`     : ユーザーが登録した食品マスタ
//! - `meal_log`      : 日付・スロットごとの食事記録（1 日 1 スロット = 1 行）
//! - `meal_log_item` : meal_log に紐付く食品エントリ（1 食品 = 1 行）
//! - `weight_log`    : 日別体重記録
//! - `settings`      : アプリ設定の KV ストア

pub mod schema;
pub mod seed;

use crate::domain::{DaySummary, Exercise, ExerciseDraft, FoodDraft, FoodItem, LoggedItem, MealTemplate, MealTemplateItem, ReportHistory, Result, Slot, Summary, TrainingSession, TrainingSet, Unit};
use rusqlite::{params, Connection};

/// DB 接続を保持し、アプリ全体で共有される。
pub struct Db {
    conn: Connection,
}

impl Db {
    /// DB ファイルを開き、スキーマ初期化とシード投入を行う。
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        schema::init_db(&conn)?;
        seed::seed_if_empty(&conn)?;
        Ok(Db { conn })
    }

    // ════════════════════════════════════════════════════════════════════════
    // 設定 (settings テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// キーに対応する設定値を取得する。未設定なら `None`。
    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [key],
                |r| r.get(0),
            )
            .ok()
    }

    /// 設定値を追加または上書きする（UPSERT）。
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════════
    // 食品マスタ (food_item テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// 全食品を名前順で取得する。
    pub fn list_foods(&self) -> Result<Vec<FoodItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, unit, kcal_per_unit, p_per_unit, f_per_unit, c_per_unit
             FROM food_item
             ORDER BY name COLLATE NOCASE",
        )?;

        let items = stmt
            .query_map([], |r| {
                Ok(FoodItem {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    unit: Unit::from_str(&r.get::<_, String>(2)?),
                    kcal_per_unit: r.get(3)?,
                    p_per_unit: r.get(4)?,
                    f_per_unit: r.get(5)?,
                    c_per_unit: r.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(items)
    }

    /// 新しい食品を登録する。名前が重複するとエラー。
    pub fn add_food(&self, draft: &FoodDraft) -> Result<()> {
        self.conn.execute(
            "INSERT INTO food_item (name, unit, kcal_per_unit, p_per_unit, f_per_unit, c_per_unit)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                draft.name,
                draft.unit.as_str(),
                draft.kcal_f(),
                draft.p_f(),
                draft.f_f(),
                draft.c_f(),
            ],
        )?;
        Ok(())
    }

    /// 既存食品の情報を更新する。
    pub fn update_food(&self, id: i64, draft: &FoodDraft) -> Result<()> {
        self.conn.execute(
            "UPDATE food_item
             SET name=?1, unit=?2, kcal_per_unit=?3, p_per_unit=?4, f_per_unit=?5, c_per_unit=?6
             WHERE id=?7",
            params![
                draft.name,
                draft.unit.as_str(),
                draft.kcal_f(),
                draft.p_f(),
                draft.f_f(),
                draft.c_f(),
                id,
            ],
        )?;
        Ok(())
    }

    /// 食品を削除する。関連する meal_log_item も CASCADE で削除される。
    pub fn delete_food(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM food_item WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 登録済みの食事ショートカットを取得する。
    pub fn list_meal_templates(&self) -> Result<Vec<MealTemplate>> {
        let mut stmt = self.conn.prepare(
            "SELECT mt.id, mt.name, fi.id, fi.name, fi.unit, mti.amount
             FROM meal_template mt
             LEFT JOIN meal_template_item mti ON mti.template_id = mt.id
             LEFT JOIN food_item fi ON fi.id = mti.food_item_id
             ORDER BY mt.name COLLATE NOCASE, mti.id",
        )?;

        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, Option<f64>>(5)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut templates: Vec<MealTemplate> = Vec::new();
        for (template_id, template_name, food_id, food_name, food_unit, amount) in rows {
            let needs_new = templates
                .last()
                .map(|t| t.id != template_id)
                .unwrap_or(true);
            if needs_new {
                templates.push(MealTemplate {
                    id: template_id,
                    name: template_name,
                    items: Vec::new(),
                });
            }

            if let (Some(_food_id), Some(food_name), Some(food_unit), Some(amount)) =
                (food_id, food_name, food_unit, amount)
            {
                templates.last_mut().unwrap().items.push(MealTemplateItem {
                    food_name,
                    unit: Unit::from_str(&food_unit),
                    amount,
                });
            }
        }

        Ok(templates)
    }

    /// 食事ショートカットを保存する。
    pub fn add_meal_template(&self, name: &str, items: &[(i64, f64)]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("INSERT INTO meal_template (name) VALUES (?1)", [name])?;
        let template_id = tx.last_insert_rowid();

        for (food_id, amount) in items {
            tx.execute(
                "INSERT INTO meal_template_item (template_id, food_item_id, amount)
                 VALUES (?1, ?2, ?3)",
                params![template_id, food_id, amount],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// 食事ショートカットを削除する。
    pub fn delete_meal_template(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM meal_template WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 食事ショートカットの構成食材を meal_log にまとめて追加する。
    pub fn add_meal_template_to_log(&self, meal_log_id: i64, template_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meal_log_item (meal_log_id, food_item_id, amount)
             SELECT ?1, food_item_id, amount
             FROM meal_template_item
             WHERE template_id = ?2",
            params![meal_log_id, template_id],
        )?;
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════════
    // 食事記録 (meal_log / meal_log_item テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// 指定日・スロットの meal_log 行を取得し、なければ作成して ID を返す。
    /// `UNIQUE(date, slot)` 制約により 1 日 1 スロット 1 行を保証する。
    pub fn get_or_create_meal_log(&self, date: &str, slot: Slot) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO meal_log (date, slot) VALUES (?1, ?2)",
            params![date, slot.as_str()],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM meal_log WHERE date = ?1 AND slot = ?2",
            params![date, slot.as_str()],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// meal_log に食品エントリを追加する。
    pub fn add_meal_item(&self, meal_log_id: i64, food_id: i64, amount: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meal_log_item (meal_log_id, food_item_id, amount)
             VALUES (?1, ?2, ?3)",
            params![meal_log_id, food_id, amount],
        )?;
        Ok(())
    }

    /// meal_log_item を削除する。
    pub fn delete_meal_item(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM meal_log_item WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 指定日のすべての食品エントリを (Slot, LoggedItem) の形で返す。
    /// meal_log → meal_log_item → food_item を JOIN して取得する。
    pub fn get_today_items(&self, date: &str) -> Result<Vec<(Slot, LoggedItem)>> {
        let mut stmt = self.conn.prepare(
            "SELECT ml.slot,
                    mli.id,
                    fi.name,
                    fi.unit,
                    mli.amount,
                    fi.kcal_per_unit * mli.amount,
                    fi.p_per_unit    * mli.amount,
                    fi.f_per_unit    * mli.amount,
                    fi.c_per_unit    * mli.amount
             FROM meal_log      ml
             JOIN meal_log_item mli ON mli.meal_log_id  = ml.id
             JOIN food_item     fi  ON fi.id            = mli.food_item_id
             WHERE ml.date = ?1
             ORDER BY ml.slot, mli.id",
        )?;

        let items = stmt
            .query_map([date], |r| {
                Ok((
                    Slot::from_str(&r.get::<_, String>(0)?),
                    LoggedItem {
                        id:        r.get(1)?,
                        food_name: r.get(2)?,
                        unit:      Unit::from_str(&r.get::<_, String>(3)?),
                        amount:    r.get(4)?,
                        kcal:      r.get(5)?,
                        p:         r.get(6)?,
                        f:         r.get(7)?,
                        c:         r.get(8)?,
                    },
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(items)
    }

    /// 指定日の合計栄養素（カロリー・PFC）を返す。
    pub fn get_today_summary(&self, date: &str) -> Result<Summary> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(fi.kcal_per_unit * mli.amount), 0),
                        COALESCE(SUM(fi.p_per_unit    * mli.amount), 0),
                        COALESCE(SUM(fi.f_per_unit    * mli.amount), 0),
                        COALESCE(SUM(fi.c_per_unit    * mli.amount), 0)
                 FROM meal_log      ml
                 JOIN meal_log_item mli ON mli.meal_log_id  = ml.id
                 JOIN food_item     fi  ON fi.id            = mli.food_item_id
                 WHERE ml.date = ?1",
                [date],
                |r| {
                    Ok(Summary {
                        kcal: r.get(0)?,
                        p:    r.get(1)?,
                        f:    r.get(2)?,
                        c:    r.get(3)?,
                    })
                },
            )
            .map_err(Into::into)
    }

    // ════════════════════════════════════════════════════════════════════════
    // 体重記録 (weight_log テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// 指定日の体重を追加または上書きする（UPSERT）。
    pub fn upsert_weight(&self, date: &str, kg: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO weight_log (date, weight_kg) VALUES (?1, ?2)
             ON CONFLICT(date) DO UPDATE SET weight_kg = excluded.weight_kg",
            params![date, kg],
        )?;
        Ok(())
    }

    /// 指定日の体重を取得する。未記録なら `None`。
    pub fn get_today_weight(&self, date: &str) -> Option<f64> {
        self.conn
            .query_row(
                "SELECT weight_kg FROM weight_log WHERE date = ?1",
                [date],
                |r| r.get(0),
            )
            .ok()
    }

    /// 直近 `days` 日分の体重履歴を日付昇順で返す。グラフ描画で使用。
    pub fn get_weight_history(&self, days: u32) -> Result<Vec<(String, f64)>> {
        let offset = format!("-{} days", days);
        let mut stmt = self.conn.prepare(
            "SELECT date, weight_kg
             FROM weight_log
             WHERE date >= date('now', ?1)
             ORDER BY date",
        )?;

        let items = stmt
            .query_map([&offset], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(items)
    }

    // ════════════════════════════════════════════════════════════════════════
    // 週次集計（レポート用）
    // ════════════════════════════════════════════════════════════════════════

    // ════════════════════════════════════════════════════════════════════════
    // LLM レポート履歴 (report_history テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// AI レポートを保存する。生成日時は呼び出し時の現在時刻を使う。
    pub fn save_report(&self, text: &str) -> Result<()> {
        let created_at = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
        self.conn.execute(
            "INSERT INTO report_history (created_at, report_text) VALUES (?1, ?2)",
            rusqlite::params![created_at, text],
        )?;
        Ok(())
    }

    /// 過去の AI レポートを新しい順で最大 50 件取得する。
    pub fn get_report_history(&self) -> Result<Vec<ReportHistory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, report_text
             FROM report_history
             ORDER BY id DESC
             LIMIT 50",
        )?;

        let items = stmt
            .query_map([], |r| {
                Ok(ReportHistory {
                    id:         r.get(0)?,
                    created_at: r.get(1)?,
                    text:       r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(items)
    }

    // ════════════════════════════════════════════════════════════════════════
    // 種目マスタ (exercise テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// 全種目を名前順で取得する。
    pub fn list_exercises(&self) -> Result<Vec<Exercise>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, muscle_group, notes FROM exercise ORDER BY name COLLATE NOCASE",
        )?;
        let items = stmt
            .query_map([], |r| {
                Ok(Exercise {
                    id:           r.get(0)?,
                    name:         r.get(1)?,
                    muscle_group: r.get(2)?,
                    notes:        r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(items)
    }

    /// 種目を追加する。名前が重複するとエラー。
    pub fn add_exercise(&self, draft: &ExerciseDraft) -> Result<()> {
        self.conn.execute(
            "INSERT INTO exercise (name, muscle_group, notes) VALUES (?1, ?2, ?3)",
            params![draft.name, draft.muscle_group, draft.notes],
        )?;
        Ok(())
    }

    /// 既存種目を更新する。
    pub fn update_exercise(&self, id: i64, draft: &ExerciseDraft) -> Result<()> {
        self.conn.execute(
            "UPDATE exercise SET name=?1, muscle_group=?2, notes=?3 WHERE id=?4",
            params![draft.name, draft.muscle_group, draft.notes, id],
        )?;
        Ok(())
    }

    /// 種目を削除する。関連セットも CASCADE で削除される。
    pub fn delete_exercise(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM exercise WHERE id = ?1", [id])?;
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════════
    // トレーニングセッション (training_session テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// 指定日のセッションを取得する。なければ作成して ID を返す。
    pub fn get_or_create_today_session(&self, date: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO training_session (date) VALUES (?1)",
            [date],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM training_session WHERE date = ?1 ORDER BY id LIMIT 1",
            [date],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    /// 指定日のセッションを取得する。なければ `None`。
    pub fn get_session_by_date(&self, date: &str) -> Result<Option<TrainingSession>> {
        let result = self.conn.query_row(
            "SELECT id, date, memo FROM training_session WHERE date = ?1 ORDER BY id LIMIT 1",
            [date],
            |r| {
                Ok(TrainingSession {
                    id:   r.get(0)?,
                    date: r.get(1)?,
                    memo: r.get(2)?,
                })
            },
        );
        match result {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// セッションのメモを更新する。
    pub fn update_session_memo(&self, session_id: i64, memo: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE training_session SET memo=?1 WHERE id=?2",
            params![memo, session_id],
        )?;
        Ok(())
    }

    // ════════════════════════════════════════════════════════════════════════
    // セット記録 (training_set テーブル)
    // ════════════════════════════════════════════════════════════════════════

    /// セットを追加する。
    pub fn add_training_set(
        &self,
        session_id: i64,
        exercise_id: i64,
        set_number: i32,
        reps: i32,
        weight_kg: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO training_set (session_id, exercise_id, set_number, reps, weight_kg)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, exercise_id, set_number, reps, weight_kg],
        )?;
        Ok(())
    }

    /// セットを削除する。
    pub fn delete_training_set(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM training_set WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 指定セッションの全セットを種目 JOIN で取得する。
    pub fn get_session_sets(&self, session_id: i64) -> Result<Vec<TrainingSet>> {
        let mut stmt = self.conn.prepare(
            "SELECT ts.id, ts.session_id, ts.exercise_id, e.name, e.muscle_group,
                    ts.set_number, ts.reps, ts.weight_kg
             FROM training_set ts
             JOIN exercise e ON e.id = ts.exercise_id
             WHERE ts.session_id = ?1
             ORDER BY e.name, ts.set_number",
        )?;
        let items = stmt
            .query_map([session_id], |r| {
                Ok(TrainingSet {
                    id:            r.get(0)?,
                    session_id:    r.get(1)?,
                    exercise_id:   r.get(2)?,
                    exercise_name: r.get(3)?,
                    muscle_group:  r.get(4)?,
                    set_number:    r.get(5)?,
                    reps:          r.get(6)?,
                    weight_kg:     r.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(items)
    }

    /// 種目の最大重量推移を返す（グラフ用）。
    pub fn get_exercise_max_weight_history(
        &self,
        exercise_id: i64,
        days: u32,
    ) -> Result<Vec<(String, f64)>> {
        let offset = format!("-{} days", days);
        let mut stmt = self.conn.prepare(
            "SELECT ts.date, MAX(t.weight_kg)
             FROM training_set t
             JOIN training_session ts ON ts.id = t.session_id
             WHERE t.exercise_id = ?1 AND ts.date >= date('now', ?2)
             GROUP BY ts.date
             ORDER BY ts.date",
        )?;
        let items = stmt
            .query_map(params![exercise_id, offset], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(items)
    }

    /// 過去 N 日のセッション一覧（セット付き）を返す（履歴画面用）。
    pub fn get_training_history(
        &self,
        days: u32,
    ) -> Result<Vec<(TrainingSession, Vec<TrainingSet>)>> {
        let offset = format!("-{} days", days);
        let mut sess_stmt = self.conn.prepare(
            "SELECT id, date, memo FROM training_session
             WHERE date >= date('now', ?1)
             ORDER BY date DESC",
        )?;
        let sessions = sess_stmt
            .query_map([&offset], |r| {
                Ok(TrainingSession {
                    id:   r.get(0)?,
                    date: r.get(1)?,
                    memo: r.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut result = Vec::new();
        for sess in sessions {
            let sets = self.get_session_sets(sess.id)?;
            result.push((sess, sets));
        }
        Ok(result)
    }

    /// 週次トレーニングサマリを返す（AIプロンプト用）。
    /// (date, total_sets, total_volume, memo)
    pub fn get_weekly_training_summary(&self) -> Result<Vec<(String, i32, f64, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT ts.date,
                    COUNT(t.id),
                    COALESCE(SUM(t.reps * t.weight_kg), 0),
                    ts.memo
             FROM training_session ts
             LEFT JOIN training_set t ON t.session_id = ts.id
             WHERE ts.date >= date('now', '-6 days')
             GROUP BY ts.id, ts.date, ts.memo
             ORDER BY ts.date",
        )?;
        let items = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i32>(1)?,
                    r.get::<_, f64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(items)
    }

    /// 直近 7 日間の日別食事合計を返す。LLM プロンプト生成とレポート画面で使用。
    pub fn get_weekly_summaries(&self) -> Result<Vec<DaySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT ml.date,
                    COALESCE(SUM(fi.kcal_per_unit * mli.amount), 0),
                    COALESCE(SUM(fi.p_per_unit    * mli.amount), 0),
                    COALESCE(SUM(fi.f_per_unit    * mli.amount), 0),
                    COALESCE(SUM(fi.c_per_unit    * mli.amount), 0)
             FROM meal_log      ml
             LEFT JOIN meal_log_item mli ON mli.meal_log_id  = ml.id
             LEFT JOIN food_item     fi  ON fi.id            = mli.food_item_id
             WHERE ml.date >= date('now', '-6 days')
             GROUP BY ml.date
             ORDER BY ml.date",
        )?;

        let items = stmt
            .query_map([], |r| {
                Ok(DaySummary {
                    date: r.get(0)?,
                    kcal: r.get(1)?,
                    p:    r.get(2)?,
                    f:    r.get(3)?,
                    c:    r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(items)
    }
}
