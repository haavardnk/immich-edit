use chrono::Utc;
use sqlx::{Row, Sqlite, Transaction};

use super::*;

impl AuthStore {
    pub async fn upsert_user(&self, user: &ImmichUser) -> Result<UserRecord, AuthStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, email, name, is_admin, access_enabled, created_at, last_login_at) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5) \
             ON CONFLICT(id) DO UPDATE SET \
               email = excluded.email, \
               name = excluded.name, \
               is_admin = excluded.is_admin, \
               last_login_at = excluded.last_login_at",
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.is_admin as i64)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        self.get_user(user.id)
            .await?
            .ok_or(AuthStoreError::Db(sqlx::Error::RowNotFound))
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<UserRecord>, AuthStoreError> {
        let row = sqlx::query(
            "SELECT id, email, name, is_admin, access_enabled FROM users WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(user_from_row).transpose()
    }

    pub async fn list_users(&self) -> Result<Vec<UserRecord>, AuthStoreError> {
        let rows = sqlx::query(
            "SELECT id, email, name, is_admin, access_enabled FROM users ORDER BY email",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(user_from_row).collect()
    }

    pub async fn set_access(&self, id: Uuid, enabled: bool) -> Result<(), AuthStoreError> {
        sqlx::query("UPDATE users SET access_enabled = ?2 WHERE id = ?1")
            .bind(id.to_string())
            .bind(enabled as i64)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub(super) async fn insert_user(
        tx: &mut Transaction<'_, Sqlite>,
        user: &ImmichUser,
    ) -> Result<UserRecord, AuthStoreError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, email, name, is_admin, access_enabled, created_at, last_login_at) \
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
        )
        .bind(user.id.to_string())
        .bind(&user.email)
        .bind(&user.name)
        .bind(user.is_admin as i64)
        .bind(&now)
        .execute(&mut **tx)
        .await?;
        Ok(UserRecord {
            id: user.id,
            email: user.email.clone(),
            name: user.name.clone(),
            is_admin: user.is_admin,
            access_enabled: true,
        })
    }

    pub async fn purge_all(&self) -> Result<(), AuthStoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM users").execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}

fn user_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<UserRecord, AuthStoreError> {
    Ok(UserRecord {
        id: parse_uuid(row.try_get::<String, _>("id")?)?,
        email: row.try_get("email")?,
        name: row.try_get("name")?,
        is_admin: row.try_get::<i64, _>("is_admin")? != 0,
        access_enabled: row.try_get::<i64, _>("access_enabled")? != 0,
    })
}
