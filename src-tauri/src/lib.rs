
use modules::objects::Keys;
use once_cell::sync::OnceCell;
use pqc_dilithium::*;
use ring::{
    rand::{SecureRandom, SystemRandom},
    signature::{Ed25519KeyPair, KeyPair},
};
use std::env;
use std::{
    collections::HashMap,
    sync::Arc,
};
use tauri::{AppHandle, Manager as _};
use tauri::Wry;
use tokio::{net::TcpStream, sync::Mutex};
mod modules;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, prelude::FromRow, sqlite::SqlitePoolOptions, Pool, Sqlite};
use sqlx::Row;
use uuid::Uuid;

pub static GLOBAL_STORE: OnceCell<Mutex<Arc<tauri_plugin_store::Store<Wry>>>> = OnceCell::new();
pub static PROFILE_NAME: once_cell::sync::Lazy<Mutex<String>> = once_cell::sync::Lazy::new(|| Mutex::new(String::new()));

lazy_static::lazy_static! {
    pub static ref SHARED_SECRETS: Arc<Mutex<HashMap<String, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
}
lazy_static::lazy_static! {
    pub static ref ENCRYPTION_KEY: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
}
lazy_static::lazy_static! {
    pub static ref NODE_SHARED_SECRET: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
}
lazy_static::lazy_static! {
    pub static ref GLOBAL_WRITE_HALF: Arc<Mutex<Option<tokio::io::WriteHalf<TcpStream>>>> = Arc::new(Mutex::new(None));
}
lazy_static::lazy_static! {
    pub static ref KEYS : Arc<Mutex<Option<modules::objects::Keys>>> = Arc::new(Mutex::new(None));
}
pub static GLOBAL_DB: OnceCell<Pool<Sqlite>> = OnceCell::new();


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chat {
    chat_id: String,
    chat_name: String,
    dst_user_id: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    message_id: String,
    chat_id: String,
    sender_id: String,
    message_type: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct KeysResponse {
    dilithium_public: Vec<u8>,
    dilithium_private: Vec<u8>,
    kyber_public: Vec<u8>,
    kyber_private: Vec<u8>,
    ed25519: Vec<u8>,
    nonce: Vec<u8>,
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct KeyStorage {
    dilithium_public: String,
    dilithium_private: String,
    kyber_public: String,
    kyber_private: String,
    ed25519: String,
    nonce: String,
    user_id: String,
    password_hash: String,
}

fn generate_pbkdf2_key(password: &str) -> Vec<u8>{
    let iterations = std::num::NonZeroU32::new(100_000).unwrap().get();

    const FIXED_SALT: &[u8] = b"this is my fixed salt!";

    let mut pbkdf2_key = [0u8; 32];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(&password.as_bytes(), &FIXED_SALT, iterations, &mut pbkdf2_key).unwrap();
    pbkdf2_key.to_vec()
}

async fn load_keys(password: &str) -> Result<Keys, String> {

    let current_profile = modules::utils::get_profile_name().await;
    let db = GLOBAL_DB
    .get()
    .ok_or_else(|| "Database not initialized".to_string())?;

    let row = sqlx::query(
        "SELECT dilithium_public, dilithium_private, kyber_public, kyber_private, ed25519, nonce, user_id, password_hash 
         FROM profiles 
         WHERE profile_name = ?"
    )
    .bind(current_profile)
    .fetch_one(db)
    .await
    .map_err(|e| format!("Error fetching profile data: {}", e))?;

    let mut key = ENCRYPTION_KEY.lock().await;
    *key = generate_pbkdf2_key(password);

    let dilithium_public: Vec<u8> = modules::encryption::decrypt_data(&row.get("dilithium_public"), &key).await.unwrap();
    let dilithium_private: Vec<u8> = modules::encryption::decrypt_data(&row.get("dilithium_private"), &key).await.unwrap();
    let kyber_public: Vec<u8> = modules::encryption::decrypt_data(&row.get("kyber_public"), &key).await.unwrap();
    let kyber_private: Vec<u8> = modules::encryption::decrypt_data(&row.get("kyber_private"), &key).await.unwrap();
    let ed25519: Vec<u8> = modules::encryption::decrypt_data(&row.get("ed25519"), &key).await.unwrap();
    let nonce: Vec<u8> = modules::encryption::decrypt_data(&row.get("nonce"), &key).await.unwrap();
    let user_id: String = row.get("user_id");
    let password_hash: String = row.get("password_hash");
    

    

    let mut d_public_key_array = [0u8; 1952];
    let mut d_private_key_array = [0u8; 4000];
    let mut nonce_array = [0u8; 16];
    let mut k_public_key_array = [0u8; 1568];
    let mut k_private_key_array = [0u8; 3168];

    d_public_key_array.copy_from_slice(&dilithium_public);
    d_private_key_array.copy_from_slice(&dilithium_private);
    nonce_array.copy_from_slice(&nonce);
    k_public_key_array.copy_from_slice(&kyber_public);
    k_private_key_array.copy_from_slice(&kyber_private);

    let dilithium_keypair = pqc_dilithium::Keypair::load(d_public_key_array, d_private_key_array);
    let ed25519: Ed25519KeyPair = Ed25519KeyPair::from_pkcs8(&ed25519).unwrap();
    let kyber_keys = pqc_kyber::Keypair {
        public: k_public_key_array,
        secret: k_private_key_array,
    };

    let is_valid = bcrypt::verify(password, &password_hash).expect("Failed to verify password");
    if is_valid {
        println!("Password is valid!");
    } else {
        println!("Invalid password!");
    }
    let keys = Keys {
        ed25519_keys: ed25519,
        dilithium_keys: dilithium_keypair,
        kyber_keys,
        nonce: nonce_array,
    };
    Ok(keys)
}

#[tauri::command]
async fn establish_ss(dst_user_id: String) {
    modules::tcp::send_kyber_key(hex::decode(dst_user_id).unwrap()).await;
}

#[tauri::command]
async fn generate_dilithium_keys(app: tauri::AppHandle, password: &str) -> Result<(), String> {
    match load_keys(&password).await {
        Ok(keys) => {
            let full_hash_input = [
                &keys.dilithium_keys.public[..],
                &keys.ed25519_keys.public_key().as_ref()[..],
                &keys.nonce[..],
            ]
            .concat();
            let user_id = modules::utils::create_user_id_hash(&full_hash_input);
            println!("{}", user_id);

            {
                let mut keys_lock = KEYS.lock().await;
                *keys_lock = Some(keys);
            }
            let app_clone = app.clone();
            tokio::spawn(async move {
                let _ = modules::tcp::server_connect(&app_clone).await;
            });
            return Ok(());
        }
        _ => println!("error"),
    }
    let rng = SystemRandom::new();
    let mut kyber_rng = rand::rngs::OsRng;
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
        .map_err(|_| "Failed to generate Ed25519 key pair".to_string())?;
    let ed25519_keys: Ed25519KeyPair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
        .map_err(|_| "Failed to parse Ed25519 key pair".to_string())?;

    let dilithium_keys: Keypair = Keypair::generate();
    let mut nonce = [0u8; 16];
    SecureRandom::fill(&rng, &mut nonce)
        .map_err(|_| "Failed to generate random bytes".to_string())?;

    let kyber_keys = pqc_kyber::keypair(&mut kyber_rng).unwrap();

    let full_hash_input = [
        &dilithium_keys.public[..],
        &ed25519_keys.public_key().as_ref()[..],
        &nonce[..],
    ]
    .concat();
    let user_id = modules::utils::create_user_id_hash(&full_hash_input);
    println!("{}", user_id);
    let keys = modules::objects::Keys {
        dilithium_keys,
        ed25519_keys,
        kyber_keys,
        nonce,
    };
    
    {
        let mut keys_lock = KEYS.lock().await;
        *keys_lock = Some(keys);
    }

    let app_clone = app.clone();

    tokio::spawn(async move {
        let _ = modules::tcp::server_connect(&app_clone).await;
    });

    let db = GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;
    let current_profile = modules::utils::get_profile_name().await;
    let hashed_password = bcrypt::hash(password, bcrypt::DEFAULT_COST).expect("Failed to hash password");
    {
        let mut key = ENCRYPTION_KEY.lock().await;
        *key = generate_pbkdf2_key(password);
    
    sqlx::query(
        "UPDATE profiles 
         SET dilithium_public = ?, 
            dilithium_private = ?, 
            kyber_public = ?,
            kyber_private = ?,
            ed25519 = ?,
            nonce = ?,
            user_id = ?,
            password_hash = ?
         WHERE profile_name = ?"
    )
        .bind(modules::encryption::encrypt_data(&dilithium_keys.public, &key).await)
        .bind(modules::encryption::encrypt_data(dilithium_keys.expose_secret(), &key).await)
        .bind(modules::encryption::encrypt_data(&kyber_keys.public, &key).await)
        .bind(modules::encryption::encrypt_data(&kyber_keys.secret, &key).await)
        .bind(modules::encryption::encrypt_data(pkcs8_bytes.as_ref(), &key).await)
        .bind(modules::encryption::encrypt_data(&nonce, &key).await)
        .bind(user_id)
        .bind(hashed_password)
        .bind(current_profile)
        .execute(db)
        .await
        .map_err(|e| format!("Error updating shared secret: {}", e))?;
    }
    Ok(())
}

async fn setup_app_state(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_db(app).await;
    GLOBAL_DB
        .set(db.clone())
        .expect("Failed to set global DB. It may have been set already.");
    app.manage(modules::objects::AppState { db });
    println!("Successfully initialised DB");
    Ok(())
}

pub async fn setup_db(app: &AppHandle) -> modules::objects::Db {
    let mut path = app.path().app_data_dir().expect("failed to get data_dir");
    println!("{:?}", &path);

    match std::fs::create_dir_all(path.clone()) {
        Ok(_) => {}
        Err(err) => {
            panic!("error creating directory {}", err);
        }
    };

    path.push("db.sqlite");

    Sqlite::create_database(
        format!(
            "sqlite:{}",
            path.to_str().expect("path should be something")
        )
        .as_str(),
    )
    .await
    .expect("failed to create database");

    let db = SqlitePoolOptions::new()
        .connect(path.to_str().unwrap())
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&db).await.unwrap();

    db
}

#[tauri::command]
async fn get_chats(state: tauri::State<'_, modules::objects::AppState>) -> Result<Vec<Chat>, String> {
    let db = &state.db;
    let current_profile = modules::utils::get_profile_name().await;
    let chats: Vec<Chat> = sqlx::query_as::<_, Chat>("SELECT * FROM chats WHERE chat_profil = ?1 ORDER BY last_updated DESC")
        .bind(current_profile)
        .fetch(db)
        .try_collect()
        .await
        .map_err(|e| format!("Failed to get chats: {}", e))?;

    Ok(chats)
}

#[tauri::command]
async fn save_message(
    state: tauri::State<'_, modules::objects::AppState>,
    sender_id: &str,
    message: String,
    message_type: &str,
) -> Result<(), String> {
    let db = &state.db;
    let key = ENCRYPTION_KEY.lock().await;

    let encrypted_message_vec = modules::encryption::encrypt_message(&message, &key).await;
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

    let message_id = Uuid::new_v4().to_string();
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
async fn get_messages(
    state: tauri::State<'_, modules::objects::AppState>,
    chat_id: &str,
) -> Result<Vec<Message>, String> {
    let db = &state.db;
    
    let key = ENCRYPTION_KEY.lock().await;
    
    let messages: Vec<Message> =
        sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE chat_id = ?1")
            .bind(chat_id)
            .fetch(db)
            .try_collect()
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        let mut decrypted_messages = Vec::new();
        for mut msg in messages {
            let encrypted_buffer = hex::decode(msg.content).unwrap();
    
            match modules::encryption::decrypt_message(&encrypted_buffer, &key).await {
                Ok(decrypted) => {
                    msg.content = decrypted;
                    decrypted_messages.push(msg);
                }
                Err(_) => return Err("Decryption failed".into()),
            }
        }
    Ok(decrypted_messages)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            generate_dilithium_keys,
            modules::tcp::send_message,
            modules::database::add_chat,
            get_chats,
            save_message,
            get_messages,
            modules::database::has_shared_secret,
            establish_ss,
            modules::database::set_profile_name,
            modules::database::delete_chat,
            modules::database::create_profil,
            modules::database::load_shared_secrets
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup_app_state(&app_handle).await {
                    eprintln!("Error setting up app state: {}", e);
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}