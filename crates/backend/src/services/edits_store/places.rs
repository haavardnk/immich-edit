use chrono::Utc;
use serde::Serialize;
use sqlx::AssertSqlSafe;
use uuid::Uuid;

use super::*;
use crate::immich::dto::PlaceName;

pub const MAX_SAVED_PLACES: i64 = 500;
pub const MAX_RECENT_PLACES: usize = 8;
const SAME_SPOT_METRES: f64 = 50.0;
const EARTH_RADIUS_METRES: f64 = 6_371_008.8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SavedPlace {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub created_at: String,
    pub used_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RecentPlace {
    #[sqlx(try_from = "Hyphenated")]
    pub id: Uuid,
    pub lat: f64,
    pub lon: f64,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub used_at: String,
}

#[derive(Debug)]
pub enum PlaceWrite {
    Saved(SavedPlace),
    Duplicate,
    Full,
    Missing,
}

const SAVED_COLUMNS: &str = "id, name, lat, lon, city, state, country, created_at, used_at";
const RECENT_COLUMNS: &str = "id, lat, lon, city, state, country, used_at";

pub fn distance_metres(a: Point, b: Point) -> f64 {
    let (lat_a, lat_b) = (a.lat.to_radians(), b.lat.to_radians());
    let half_lat = (lat_b - lat_a) / 2.0;
    let half_lon = (b.lon - a.lon).to_radians() / 2.0;
    let h = half_lat.sin().powi(2) + lat_a.cos() * lat_b.cos() * half_lon.sin().powi(2);
    2.0 * EARTH_RADIUS_METRES * h.sqrt().min(1.0).asin()
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .is_some_and(|e| e.is_unique_violation())
}

impl EditsStore {
    pub async fn list_saved_places(&self, owner: Uuid) -> Result<Vec<SavedPlace>, EditsStoreError> {
        Ok(sqlx::query_as::<_, SavedPlace>(AssertSqlSafe(format!(
            "SELECT {SAVED_COLUMNS} FROM saved_places WHERE user_id = ?1 \
             ORDER BY used_at DESC, name COLLATE NOCASE"
        )))
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn list_recent_places(
        &self,
        owner: Uuid,
    ) -> Result<Vec<RecentPlace>, EditsStoreError> {
        Ok(sqlx::query_as::<_, RecentPlace>(AssertSqlSafe(format!(
            "SELECT {RECENT_COLUMNS} FROM recent_places WHERE user_id = ?1 ORDER BY used_at DESC"
        )))
        .bind(owner.to_string())
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn create_saved_place(
        &self,
        owner: Uuid,
        name: &str,
        at: Point,
        place: &PlaceName,
    ) -> Result<PlaceWrite, EditsStoreError> {
        let mut tx = self.pool.begin().await?;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM saved_places WHERE user_id = ?1")
            .bind(owner.to_string())
            .fetch_one(&mut *tx)
            .await?;
        if count >= MAX_SAVED_PLACES {
            return Ok(PlaceWrite::Full);
        }
        let now = Utc::now().to_rfc3339();
        let inserted = sqlx::query_as::<_, SavedPlace>(AssertSqlSafe(format!(
            "INSERT INTO saved_places \
             (id, user_id, name, lat, lon, city, state, country, created_at, used_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9) \
             ON CONFLICT DO NOTHING RETURNING {SAVED_COLUMNS}"
        )))
        .bind(Uuid::new_v4().to_string())
        .bind(owner.to_string())
        .bind(name)
        .bind(at.lat)
        .bind(at.lon)
        .bind(&place.city)
        .bind(&place.state)
        .bind(&place.country)
        .bind(&now)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(inserted.map_or(PlaceWrite::Duplicate, PlaceWrite::Saved))
    }

    pub async fn update_saved_place(
        &self,
        owner: Uuid,
        id: Uuid,
        name: &str,
        at: Point,
        place: &PlaceName,
    ) -> Result<PlaceWrite, EditsStoreError> {
        let updated = sqlx::query_as::<_, SavedPlace>(AssertSqlSafe(format!(
            "UPDATE saved_places SET name = ?3, lat = ?4, lon = ?5, city = ?6, state = ?7, \
             country = ?8 WHERE user_id = ?2 AND id = ?1 RETURNING {SAVED_COLUMNS}"
        )))
        .bind(id.to_string())
        .bind(owner.to_string())
        .bind(name)
        .bind(at.lat)
        .bind(at.lon)
        .bind(&place.city)
        .bind(&place.state)
        .bind(&place.country)
        .fetch_optional(&self.pool)
        .await;
        match updated {
            Ok(Some(record)) => Ok(PlaceWrite::Saved(record)),
            Ok(None) => Ok(PlaceWrite::Missing),
            Err(err) if is_unique_violation(&err) => Ok(PlaceWrite::Duplicate),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn touch_saved_place(&self, owner: Uuid, id: Uuid) -> Result<bool, EditsStoreError> {
        let res =
            sqlx::query("UPDATE saved_places SET used_at = ?3 WHERE user_id = ?2 AND id = ?1")
                .bind(id.to_string())
                .bind(owner.to_string())
                .bind(Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn delete_saved_place(&self, owner: Uuid, id: Uuid) -> Result<bool, EditsStoreError> {
        let res = sqlx::query("DELETE FROM saved_places WHERE user_id = ?2 AND id = ?1")
            .bind(id.to_string())
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn remember_place(
        &self,
        owner: Uuid,
        at: Point,
        place: &PlaceName,
    ) -> Result<Vec<RecentPlace>, EditsStoreError> {
        let mut tx = self.pool.begin().await?;
        let recent = sqlx::query_as::<_, RecentPlace>(AssertSqlSafe(format!(
            "SELECT {RECENT_COLUMNS} FROM recent_places WHERE user_id = ?1 ORDER BY used_at DESC"
        )))
        .bind(owner.to_string())
        .fetch_all(&mut *tx)
        .await?;
        let same = recent
            .iter()
            .filter(|row| {
                distance_metres(
                    Point {
                        lat: row.lat,
                        lon: row.lon,
                    },
                    at,
                ) <= SAME_SPOT_METRES
            })
            .map(|row| row.id.to_string());
        for id in same {
            sqlx::query("DELETE FROM recent_places WHERE id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query(
            "INSERT INTO recent_places (id, user_id, lat, lon, city, state, country, used_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(owner.to_string())
        .bind(at.lat)
        .bind(at.lon)
        .bind(&place.city)
        .bind(&place.state)
        .bind(&place.country)
        .bind(Utc::now().to_rfc3339())
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "DELETE FROM recent_places WHERE user_id = ?1 AND id NOT IN \
             (SELECT id FROM recent_places WHERE user_id = ?1 ORDER BY used_at DESC LIMIT ?2)",
        )
        .bind(owner.to_string())
        .bind(MAX_RECENT_PLACES as i64)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.list_recent_places(owner).await
    }

    pub async fn clear_recent_places(&self, owner: Uuid) -> Result<(), EditsStoreError> {
        sqlx::query("DELETE FROM recent_places WHERE user_id = ?1")
            .bind(owner.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_matches_known_spans() {
        let bergen = Point {
            lat: 60.3913,
            lon: 5.3221,
        };
        let oslo = Point {
            lat: 59.9139,
            lon: 10.7522,
        };
        let span = distance_metres(bergen, oslo);
        assert!((span - 305_000.0).abs() < 2_000.0, "{span}");
        let nearby = Point {
            lat: 60.3916,
            lon: 5.3221,
        };
        assert!((distance_metres(bergen, nearby) - 33.4).abs() < 0.5);
        assert_eq!(distance_metres(bergen, bergen), 0.0);
    }
}
