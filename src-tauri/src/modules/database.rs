use futures::TryStreamExt;

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
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Profile {
    pub profile_id: String,
    pub profile_name: String,
}

#[tauri::command]
pub async fn get_profiles() -> Result<Vec<Profile>, String> {
    let db = super::super::GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let chats: Vec<Profile> = sqlx::query_as::<_, Profile>("SELECT  profile_id, profile_name FROM profiles")
        .fetch_all(db)
        .await
        .map_err(|e| format!("Failed to get profiles: {}", e))?;

    Ok(chats)
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

#[tauri::command]
pub async fn save_message(
    state: tauri::State<'_, super::objects::AppState>,
    sender_id: &str,
    message: String,
    message_type: &str,
) -> Result<(), String> {
    let db = &state.db;
    let key = super::super::ENCRYPTION_KEY.lock().await;

    let encrypted_message_vec = super::encryption::encrypt_message(&message, &key).await;
    let encrypted_message = hex::encode(encrypted_message_vec);

    let current_time = chrono::Utc::now().timestamp();

    let chat_id: String = sqlx::query_scalar("SELECT chat_id FROM chats WHERE dst_user_id = ?")
        .bind(sender_id)
        .fetch_one(db)
        .await
        .map_err(|e| format!("Failed to get chat_id: {}", e))?;

    sqlx::query("UPDATE chats SET last_updated = ? WHERE chat_id = ?")
        .bind(&current_time)
        .bind(&chat_id)
        .execute(db)
        .await
        .map_err(|e| format!("Error updating shared secret: {}", e))?;

    let message_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO messages (message_id, sender_id, message_type, content, chat_id) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(message_id)
        .bind(sender_id)
        .bind(message_type)
        .bind(encrypted_message)
        .bind(chat_id)
        .execute(db)
        .await
        .map_err(|e| format!("Error saving todo: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_messages(
    state: tauri::State<'_, super::objects::AppState>,
    chat_id: &str,
) -> Result<Vec<super::objects::Message>, String> {
    let db = &state.db;
    
    let key = super::super::ENCRYPTION_KEY.lock().await;
    
    let messages: Vec<super::objects::Message> =
        sqlx::query_as::<_, super::objects::Message>("SELECT * FROM messages WHERE chat_id = ?1")
            .bind(chat_id)
            .fetch(db)
            .try_collect()
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        let mut decrypted_messages = Vec::new();
        for mut msg in messages {
            let encrypted_buffer = hex::decode(msg.content).unwrap();
    
            match super::encryption::decrypt_message(&encrypted_buffer, &key).await {
                Ok(decrypted) => {
                    msg.content = decrypted;
                    decrypted_messages.push(msg);
                }
                Err(_) => return Err("Decryption failed".into()),
            }
        }
    Ok(decrypted_messages)
}

#[tauri::command]
pub async fn get_chats(state: tauri::State<'_, super::objects::AppState>) -> Result<Vec<super::objects::Chat>, String> {
    let db = &state.db;
    let current_profile = super::utils::get_profile_name().await;
    let chats: Vec<super::objects::Chat> = sqlx::query_as::<_, super::objects::Chat>("SELECT * FROM chats WHERE chat_profil = ?1 ORDER BY last_updated DESC")
        .bind(current_profile)
        .fetch(db)
        .try_collect()
        .await
        .map_err(|e| format!("Failed to get chats: {}", e))?;

    Ok(chats)
}
