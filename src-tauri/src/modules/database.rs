#[tauri::command]
pub async fn create_profil(name: &str) -> Result<(), String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    sqlx::query("INSERT INTO profiles (profile_name) VALUES (?1)")
        .bind(name)
        .execute(db)
        .await
        .map_err(|e| format!("Error saving chat: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn set_profile_name(name: String) {
    {
        let mut profile_name = super::super::PROFILE_NAME.lock().await;
        *profile_name = name;
    }
}

#[tauri::command]
pub async fn delete_chat(chat_id: &str) -> Result<(), String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    
    sqlx::query("DELETE FROM chats WHERE chat_id = ?")
        .bind(chat_id)
        .execute(&*db)
        .await
        .map_err(|e| format!("Error deleting chat: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn add_chat(
    state: tauri::State<'_, super::objects::AppState>,
    name: &str,
    dst_user_id: &str,
) -> Result<String, String> {
    let db = &state.db;
    let chat_id = uuid::Uuid::new_v4().to_string();
    let current_profile = super::utils::get_profile_name().await;
    let current_time = chrono::Utc::now().timestamp();
    sqlx::query("INSERT INTO chats (chat_id, chat_name, dst_user_id, last_updated, chat_profil) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(&chat_id)
        .bind(name)
        .bind(dst_user_id)
        .bind(current_time)
        .bind(current_profile)
        .execute(db)
        .await
        .map_err(|e| format!("Error saving chat: {}", e))?;

    println!("Saved chat {} for user {} successfully", name, dst_user_id);

    Ok(chat_id)
}

#[tauri::command]
pub async fn has_shared_secret(chat_id: &str) -> Result<Option<bool>, String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let result: Option<(i64, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT COUNT(*), shared_secret FROM chats WHERE chat_id = ?")
            .bind(chat_id)
            .fetch_optional(db)
            .await
            .map_err(|e| format!("Failed to get chat_id: {}", e))?;

    match result {
        Some((0, _)) => Ok(None),
        Some((_, None)) => Ok(Some(false)),
        Some((_, Some(_))) => Ok(Some(true)),
        None => Ok(None),
    }
}
#[tauri::command]
pub async fn load_shared_secrets() -> Result<(), String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let current_profile = super::utils::get_profile_name().await;
    let rows: Vec<(Vec<u8>, String)> =
        sqlx::query_as("SELECT shared_secret, dst_user_id FROM chats WHERE chat_profil = ?")
            .bind(&current_profile)
            .fetch_all(db)
            .await
            .map_err(|e| format!("Failed to load shared secrets: {}", e))?;

    let mut shared_secrets_map = super::super::SHARED_SECRETS.lock().await;

    for (shared_secret, dst_user_id) in rows {
        if shared_secret.len() != 32 {
            println!("Warning: Shared secret for user_id {} is not 32 bytes. Skipping.", dst_user_id);
            continue;
        }
        shared_secrets_map.insert(dst_user_id.clone(), shared_secret.clone());
    }

    Ok(())
}

pub async fn save_shared_secret(user_id: &str, shared_secret: Vec<u8>) -> Result<String, String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let current_profile = super::utils::get_profile_name().await;
    let chat_id: String = sqlx::query_scalar("SELECT chat_id FROM chats WHERE dst_user_id = ?1 AND chat_profil= ?2")
        .bind(user_id)
        .bind(current_profile)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to get chat_id: {}", e))?;

    sqlx::query("UPDATE chats SET shared_secret = ? WHERE chat_id = ?")
        .bind(&shared_secret)
        .bind(&chat_id)
        .execute(db)
        .await
        .map_err(|e| format!("Error updating shared secret: {}", e))?;

    println!("Successfully saved shared secret for chat_id: {}", chat_id);

    Ok(chat_id)
}