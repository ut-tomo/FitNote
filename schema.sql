PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS food_item (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  serving_g REAL,
  kcal REAL NOT NULL,
  p REAL NOT NULL,
  f REAL NOT NULL,
  c REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS template (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS template_item (
  template_id INTEGER NOT NULL,
  food_item_id INTEGER NOT NULL,
  multiplier REAL NOT NULL,
  PRIMARY KEY (template_id, food_item_id),
  FOREIGN KEY (template_id) REFERENCES template(id) ON DELETE CASCADE,
  FOREIGN KEY (food_item_id) REFERENCES food_item(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS meal_log (
  id INTEGER PRIMARY KEY,
  date TEXT NOT NULL,
  slot TEXT NOT NULL,
  template_id INTEGER,
  note TEXT,
  FOREIGN KEY (template_id) REFERENCES template(id)
);

CREATE TABLE IF NOT EXISTS meal_log_item (
  meal_log_id INTEGER NOT NULL,
  food_item_id INTEGER NOT NULL,
  multiplier REAL NOT NULL,
  PRIMARY KEY (meal_log_id, food_item_id, multiplier),
  FOREIGN KEY (meal_log_id) REFERENCES meal_log(id) ON DELETE CASCADE,
  FOREIGN KEY (food_item_id) REFERENCES food_item(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS weight_log (
  date TEXT PRIMARY KEY,
  weight_kg REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS activity_log (
  date TEXT PRIMARY KEY,
  trained INTEGER NOT NULL
);
