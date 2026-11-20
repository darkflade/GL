use crate::domain::playback::{PlaybackPreferences, PlaybackSettings};
use crate::web::error::AppError;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

#[derive(Clone)]
pub struct PostgresPlaybackRepository {
    pool: PgPool,
}
impl PostgresPlaybackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    async fn check(&self, user: Uuid, collection: Option<Uuid>) -> Result<(), AppError> {
        if let Some(id) = collection {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM collections WHERE id=$1 AND owner_id=$2)",
            )
            .bind(id)
            .bind(user)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error)?;
            if !exists {
                return Err(AppError::NotFound("Collection not found".into()));
            }
        }
        Ok(())
    }
    pub async fn get(
        &self,
        user: Uuid,
        collection: Option<Uuid>,
    ) -> Result<PlaybackPreferences, AppError> {
        self.check(user, collection).await?;
        let rows: Vec<(Option<Uuid>, Json<PlaybackSettings>)> = sqlx::query_as(
            "SELECT collection_id, settings FROM playback_settings WHERE user_id=$1 AND (collection_id IS NULL OR collection_id=$2)")
            .bind(user).bind(collection).fetch_all(&self.pool).await.map_err(db_error)?;
        let item_id: Option<Uuid> = if let Some(id) = collection {
            sqlx::query_scalar::<_, Option<Uuid>>("SELECT item_id FROM collection_playback_positions WHERE user_id=$1 AND collection_id=$2")
                .bind(user).bind(id).fetch_optional(&self.pool).await.map_err(db_error)?.flatten()
        } else {
            None
        };
        let mut result = PlaybackPreferences {
            item_id,
            global: PlaybackSettings::default(),
            collection: None,
        };
        for (id, Json(settings)) in rows {
            if id.is_none() {
                result.global = settings;
            } else {
                result.collection = Some(settings);
            }
        }
        Ok(result)
    }
    pub async fn save(
        &self,
        user: Uuid,
        collection: Option<Uuid>,
        settings: PlaybackSettings,
    ) -> Result<(), AppError> {
        self.check(user, collection).await?;
        if let Some(item) = settings.shortcut_item_id {
            let id = collection.ok_or_else(|| {
                AppError::bad_request("A shortcut belongs to a specific collection")
            })?;
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM collection_items WHERE id=$1 AND collection_id=$2)",
            )
            .bind(item)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error)?;
            if !exists {
                return Err(AppError::not_found(
                    "Shortcut item not found in this collection",
                ));
            }
        }
        sqlx::query("INSERT INTO playback_settings(user_id, collection_id, settings) VALUES ($1,$2,$3) ON CONFLICT (user_id, collection_id) DO UPDATE SET settings=EXCLUDED.settings")
            .bind(user).bind(collection).bind(Json(settings)).execute(&self.pool).await.map_err(db_error)?;
        Ok(())
    }
    pub async fn save_position(
        &self,
        user: Uuid,
        collection: Uuid,
        item: Uuid,
    ) -> Result<(), AppError> {
        self.check(user, Some(collection)).await?;
        let result = sqlx::query("INSERT INTO collection_playback_positions(user_id,collection_id,item_id) SELECT $1,$2,id FROM collection_items WHERE id=$3 AND collection_id=$2 ON CONFLICT (user_id,collection_id) DO UPDATE SET item_id=EXCLUDED.item_id")
            .bind(user).bind(collection).bind(item).execute(&self.pool).await.map_err(db_error)?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Collection item not found"));
        }
        Ok(())
    }
    pub async fn reset(&self, user: Uuid, collection: Option<Uuid>) -> Result<(), AppError> {
        self.check(user, collection).await?;
        sqlx::query("DELETE FROM playback_settings WHERE user_id=$1 AND collection_id IS NOT DISTINCT FROM $2")
            .bind(user).bind(collection).execute(&self.pool).await.map_err(db_error)?;
        Ok(())
    }
}
fn db_error(err: sqlx::Error) -> AppError {
    log::error!("playback storage: {err}");
    AppError::internal("Could not access playback settings")
}
