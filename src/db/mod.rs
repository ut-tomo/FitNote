pub mod schema;
pub mod seed;

use crate::domain::{Result, Slot, Summary};
use rusqlite::Connection;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        schema::init_db(&conn)?;
        seed::seed_if_empty(&conn)?;
        Ok(Db { conn })
    }

    /// Upsert today's weight.
    pub fn upsert_weight(&self, date: &str, kg: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO weight_log (date, weight_kg) VALUES (?1, ?2)
             ON CONFLICT(date) DO UPDATE SET weight_kg = excluded.weight_kg",
            rusqlite::params![date, kg],
        )?;
        Ok(())
    }

    /// Log a meal from a template. Returns the new meal_log id.
    pub fn log_meal_template(&self, date: &str, slot: Slot, template_name: &str) -> Result<i64> {
        // Look up template id
        let template_id: i64 = self.conn.query_row(
            "SELECT id FROM template WHERE name = ?1",
            [template_name],
            |r| r.get(0),
        )?;

        self.conn.execute(
            "INSERT INTO meal_log (date, slot, template_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![date, slot.as_str(), template_id],
        )?;
        let meal_log_id = self.conn.last_insert_rowid();
        Ok(meal_log_id)
    }

    /// Log a meal without template (e.g. Snack). Returns the new meal_log id.
    pub fn log_meal_no_template(&self, date: &str, slot: Slot) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO meal_log (date, slot, template_id) VALUES (?1, ?2, NULL)",
            rusqlite::params![date, slot.as_str()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Add a food item (topping) to an existing meal_log.
    pub fn add_item_to_meal(&self, meal_log_id: i64, food_name: &str, multiplier: f64) -> Result<()> {
        let food_id: i64 = self.conn.query_row(
            "SELECT id FROM food_item WHERE name = ?1",
            [food_name],
            |r| r.get(0),
        )?;

        self.conn.execute(
            "INSERT OR REPLACE INTO meal_log_item (meal_log_id, food_item_id, multiplier)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![meal_log_id, food_id, multiplier],
        )?;
        Ok(())
    }

    /// Get today's nutrition summary (template items + meal_log_items).
    pub fn get_today_summary(&self, date: &str) -> Result<Summary> {
        // Sum from template_item via meal_log -> template -> template_item -> food_item
        let template_sum: Summary = self.conn.query_row(
            "SELECT COALESCE(SUM(fi.kcal * ti.multiplier), 0),
                    COALESCE(SUM(fi.p * ti.multiplier), 0),
                    COALESCE(SUM(fi.f * ti.multiplier), 0),
                    COALESCE(SUM(fi.c * ti.multiplier), 0)
             FROM meal_log ml
             JOIN template_item ti ON ti.template_id = ml.template_id
             JOIN food_item fi ON fi.id = ti.food_item_id
             WHERE ml.date = ?1",
            [date],
            |r| {
                Ok(Summary {
                    kcal: r.get(0)?,
                    p: r.get(1)?,
                    f: r.get(2)?,
                    c: r.get(3)?,
                })
            },
        )?;

        // Sum from meal_log_item (toppings)
        let topping_sum: Summary = self.conn.query_row(
            "SELECT COALESCE(SUM(fi.kcal * mli.multiplier), 0),
                    COALESCE(SUM(fi.p * mli.multiplier), 0),
                    COALESCE(SUM(fi.f * mli.multiplier), 0),
                    COALESCE(SUM(fi.c * mli.multiplier), 0)
             FROM meal_log ml
             JOIN meal_log_item mli ON mli.meal_log_id = ml.id
             JOIN food_item fi ON fi.id = mli.food_item_id
             WHERE ml.date = ?1",
            [date],
            |r| {
                Ok(Summary {
                    kcal: r.get(0)?,
                    p: r.get(1)?,
                    f: r.get(2)?,
                    c: r.get(3)?,
                })
            },
        )?;

        Ok(Summary {
            kcal: template_sum.kcal + topping_sum.kcal,
            p: template_sum.p + topping_sum.p,
            f: template_sum.f + topping_sum.f,
            c: template_sum.c + topping_sum.c,
        })
    }
}
