use crate::domain::Result;
use rusqlite::Connection;

const SCHEMA_SQL: &str = include_str!("../../schema.sql");

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(())
}
