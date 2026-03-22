//! DB スキーマの初期化とマイグレーション
//!
//! アプリ起動時に `init_db()` を呼び出すことで、スキーマのバージョン管理と
//! 必要に応じたマイグレーションを行う。
//!
//! # マイグレーション方針
//! - v0（初回）: schema.sql 全体を実行
//! - v1: 古いバージョン → 全DROP+再作成
//! - v2→v3: report_history テーブルのみ追加（既存データは保持）
//! - v3→v4: exercise / training_session / training_set テーブル追加（既存データ保持）
//! - v4→v5: meal_template / meal_template_item テーブル追加（既存データ保持）
//! - v5: 最新 → 何もしない

use crate::domain::Result;
use rusqlite::Connection;

/// 現在のスキーマバージョン。変更時はインクリメントする。
const DB_VERSION: i64 = 5;

/// `schema.sql` をコンパイル時にバイナリへ埋め込む（初回インストール用）。
const SCHEMA_SQL: &str = include_str!("../../schema.sql");

/// v2→v3 の差分マイグレーション SQL（既存データを保持）。
const V3_MIGRATION_SQL: &str = "
CREATE TABLE IF NOT EXISTS report_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at  TEXT NOT NULL,
  report_text TEXT NOT NULL
);
";

/// v3→v4 の差分マイグレーション SQL（既存データを保持）。
const V4_MIGRATION_SQL: &str = "
CREATE TABLE IF NOT EXISTS exercise (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT    NOT NULL UNIQUE,
  muscle_group TEXT    NOT NULL DEFAULT '',
  notes        TEXT    NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS training_session (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT    NOT NULL,
  memo TEXT    NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS training_set (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  INTEGER NOT NULL,
  exercise_id INTEGER NOT NULL,
  set_number  INTEGER NOT NULL,
  reps        INTEGER NOT NULL,
  weight_kg   REAL    NOT NULL DEFAULT 0,
  FOREIGN KEY (session_id)  REFERENCES training_session(id) ON DELETE CASCADE,
  FOREIGN KEY (exercise_id) REFERENCES exercise(id)         ON DELETE CASCADE
);
";

/// v4→v5 の差分マイグレーション SQL（既存データを保持）。
const V5_MIGRATION_SQL: &str = "
CREATE TABLE IF NOT EXISTS meal_template (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS meal_template_item (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  template_id  INTEGER NOT NULL,
  food_item_id INTEGER NOT NULL,
  amount       REAL    NOT NULL,
  FOREIGN KEY (template_id)  REFERENCES meal_template(id) ON DELETE CASCADE,
  FOREIGN KEY (food_item_id) REFERENCES food_item(id)     ON DELETE CASCADE
);
";

/// DB を初期化する。バージョンに応じてマイグレーションを選択する。
pub fn init_db(conn: &Connection) -> Result<()> {
    // settings テーブルはバージョン確認に必要なので必ず先に作る
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    )?;

    let saved_version: i64 = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM settings WHERE key = 'db_version'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if saved_version == 0 {
        // 初回起動: フルスキーマを作成
        conn.execute_batch(SCHEMA_SQL)?;
    } else if saved_version < 2 {
        // 旧バージョン（v1以下）: 全テーブルを再作成
        drop_all_tables(conn)?;
        conn.execute_batch(SCHEMA_SQL)?;
    } else if saved_version == 2 {
        // v2→v3: report_history テーブルのみ追加（既存データ保持）
        conn.execute_batch(V3_MIGRATION_SQL)?;
        conn.execute_batch(V4_MIGRATION_SQL)?;
    } else if saved_version == 3 {
        // v3→v4: exercise / training_session / training_set テーブル追加（既存データ保持）
        conn.execute_batch(V4_MIGRATION_SQL)?;
        conn.execute_batch(V5_MIGRATION_SQL)?;
    } else if saved_version == 4 {
        // v4→v5: meal_template / meal_template_item テーブル追加（既存データ保持）
        conn.execute_batch(V5_MIGRATION_SQL)?;
    }
    // saved_version == DB_VERSION の場合は何もしない

    if saved_version != DB_VERSION {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('db_version', ?1)",
            [DB_VERSION.to_string()],
        )?;
    }

    Ok(())
}

/// 旧スキーマのテーブルをすべて削除する。
fn drop_all_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS meal_log_item;
         DROP TABLE IF EXISTS meal_log;
         DROP TABLE IF EXISTS template_item;
         DROP TABLE IF EXISTS template;
         DROP TABLE IF EXISTS food_item;
         DROP TABLE IF EXISTS weight_log;
         DROP TABLE IF EXISTS activity_log;
         DROP TABLE IF EXISTS report_history;
         DROP TABLE IF EXISTS settings;",
    )?;
    Ok(())
}
