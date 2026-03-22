PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS food_item (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  name           TEXT    NOT NULL UNIQUE,
  unit           TEXT    NOT NULL DEFAULT 'g',
  kcal_per_unit  REAL    NOT NULL,
  p_per_unit     REAL    NOT NULL,
  f_per_unit     REAL    NOT NULL,
  c_per_unit     REAL    NOT NULL
);

CREATE TABLE IF NOT EXISTS meal_log (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT    NOT NULL,
  slot TEXT    NOT NULL,
  UNIQUE(date, slot)
);

CREATE TABLE IF NOT EXISTS meal_log_item (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  meal_log_id  INTEGER NOT NULL,
  food_item_id INTEGER NOT NULL,
  amount       REAL    NOT NULL,
  FOREIGN KEY (meal_log_id)  REFERENCES meal_log(id)  ON DELETE CASCADE,
  FOREIGN KEY (food_item_id) REFERENCES food_item(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS weight_log (
  date      TEXT PRIMARY KEY,
  weight_kg REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS report_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at  TEXT NOT NULL,
  report_text TEXT NOT NULL
);

-- 種目マスタ
CREATE TABLE IF NOT EXISTS exercise (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  name         TEXT    NOT NULL UNIQUE,
  muscle_group TEXT    NOT NULL DEFAULT '',
  notes        TEXT    NOT NULL DEFAULT ''
);

-- トレーニングセッション（1日1件以上可）
CREATE TABLE IF NOT EXISTS training_session (
  id   INTEGER PRIMARY KEY AUTOINCREMENT,
  date TEXT    NOT NULL,
  memo TEXT    NOT NULL DEFAULT ''
);

-- セット記録（セッション × 種目 × セット番号）
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
