//! DB 初期データ投入
//!
//! このアプリでは食品はユーザーが自分で登録するため、シードデータは存在しない。
//! 将来的にデモデータやプリセットが必要になった場合はここに追加する。

use crate::domain::Result;
use rusqlite::Connection;

/// 食品テーブルが空の場合にデフォルトデータを投入する（現在は何もしない）。
pub fn seed_if_empty(_conn: &Connection) -> Result<()> {
    Ok(())
}
