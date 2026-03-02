use crate::domain::Result;
use rusqlite::Connection;

pub fn seed_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM food_item", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    conn.execute_batch(
        "
        INSERT INTO food_item (id, name, serving_g, kcal, p, f, c) VALUES
          (1, '米200g(炊後)',       200, 336, 5,    0.5, 76),
          (2, '鶏むね100g',         100, 110, 23,   1.5, 0),
          (3, '卵1個',              50,  80,  6.5,  5.5, 0.3),
          (4, '肉味噌70g',          70,  80,  16,   2,   2),
          (5, '和菓子200kcal',       0,  200, 3,    1,   45),
          (6, '牛赤身200g',         200, 340, 46,   14,  0),
          (7, '朝: オートミール',     0,  350, 35,   6,   40);

        INSERT INTO template (id, name) VALUES
          (1, '朝: オートミール'),
          (2, '昼軸: 米200+鶏むね100'),
          (3, '夜軸: 米200+鶏むね100');

        INSERT INTO template_item (template_id, food_item_id, multiplier) VALUES
          (1, 7, 1.0),
          (2, 1, 1.0),
          (2, 2, 1.0),
          (3, 1, 1.0),
          (3, 2, 1.0);
        ",
    )?;

    Ok(())
}
