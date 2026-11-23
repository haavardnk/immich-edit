use sqlx::{AssertSqlSafe, SqliteConnection};
use uuid::Uuid;

use super::*;

pub(super) const ASSET_TABLES: [&str; 6] = [
    "edits",
    "edits_history",
    "edit_snapshots",
    "raster_refs",
    "export_jobs",
    "export_outputs",
];

const OWNER_TABLES: [&str; 4] = ["presets", "saved_views", "saved_places", "recent_places"];

impl EditsStore {
    pub async fn purge_owner(&self, owner: Uuid) -> Result<(), EditsStoreError> {
        let mut tx = self.pool.begin().await?;
        for table in ASSET_TABLES.iter().chain(&OWNER_TABLES) {
            sqlx::query(AssertSqlSafe(format!(
                "DELETE FROM {table} WHERE user_id = ?1"
            )))
            .bind(owner.to_string())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn purge_owned_rows(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
        for table in ASSET_TABLES.iter().chain(&OWNER_TABLES) {
            sqlx::query(AssertSqlSafe(format!("DELETE FROM {table}")))
                .execute(&mut *conn)
                .await?;
        }
        Ok(())
    }
}
