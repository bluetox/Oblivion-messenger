use crate::ProfileSettings;
use crate::{PROFILE_NAME, GLOBAL_DB};

#[tauri::command]
pub async fn get_profile_settings() -> Result<Option<ProfileSettings>, String> {
    let profile_name = PROFILE_NAME.lock().await;

    let db = GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let result: Option<(i64, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT COUNT(*), settings FROM profiles WHERE profile_name = ?")
            .bind(&*profile_name)
            .fetch_optional(db)
            .await
            .map_err(|e| format!("Failed to fetch settings: {}", e))?;

    if let Some((count, Some(binary_settings))) = result {
        if count == 0 {
            return Ok(None);
        }

        let settings: ProfileSettings = bincode::deserialize(&binary_settings)
            .map_err(|e| format!("Failed to deserialize settings: {}", e))?;

        Ok(Some(settings))
    } else {
        Ok(None)
    }
}
