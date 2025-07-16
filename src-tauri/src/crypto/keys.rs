use super::{objects::*, utils::decrypt_data};
use crate::{crypto::keys, GLOBAL_DB};

use bip39::{Language, Mnemonic};
use hkdf::Hkdf;
use sha2::Sha256;
use sqlx::Row;

const PBKDF2_ITER: u32 = 2_000;

pub async fn load_keys(password: &str) -> Result<Keys, String> {
    let key = keys::generate_pbkdf2_key(password)?;

    let current_profile = crate::utils::get_profile_name().await;

    let db = GLOBAL_DB
        .get()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let row = sqlx::query(
        "SELECT dilithium_public, dilithium_private, ed25519, nonce, user_id, password_hash 
         FROM profiles 
         WHERE profile_name = ?",
    )
    .bind(current_profile)
    .fetch_one(db)
    .await
    .map_err(|e| format!("Error fetching profile data: {}", e))?;

    let password_hash: String = row.get("password_hash");
    let is_valid = bcrypt::verify(password, &password_hash).expect("Failed to verify password");
    if is_valid {
        println!("Password is valid!");
    } else {
        println!("Invalid password!");
    }

    let dilithium_public: Vec<u8> = decrypt_data(&row.get("dilithium_public"), &key).await?;
    let dilithium_private: Vec<u8> = decrypt_data(&row.get("dilithium_private"), &key).await?;
    let seed: Vec<u8> = decrypt_data(&row.get("ed25519"), &key).await?;
    let nonce: Vec<u8> = decrypt_data(&row.get("nonce"), &key).await?;
    let _user_id: String = row.get("user_id");

    let mut d_public_key_array = [0u8; 1952];
    let mut d_private_key_array = [0u8; 4000];
    let mut nonce_array = [0u8; 16];

    d_public_key_array.copy_from_slice(&dilithium_public);
    d_private_key_array.copy_from_slice(&dilithium_private);
    nonce_array.copy_from_slice(&nonce);

    let dilithium_keypair = DilithiumKeypair::load(d_public_key_array, d_private_key_array);
    let ed25519_keys = Ed25519KeyPair::from_seed_unchecked(&seed[32..]).unwrap();

    let keys = Keys {
        ed25519_keys: ed25519_keys,
        dilithium_keys: dilithium_keypair,
        nonce: nonce_array,
        global_key: key,
    };
    Ok(keys)
}

pub fn generate_pbkdf2_key(password: &str) -> Result<Vec<u8>, String> {
    const FIXED_SALT: &[u8] = b"this is my fixed salt!";

    let mut pbkdf2_key = [0u8; 32];
    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        FIXED_SALT,
        PBKDF2_ITER,
        &mut pbkdf2_key,
    )
    .map_err(|_| "Failed to generate pbkdf2 key")?;
    Ok(pbkdf2_key.to_vec())
}

#[tauri::command]
pub fn generate_mnemonic() -> Result<Vec<String>, String> {
    let mnemonic = Mnemonic::generate_in(Language::English, 24).map_err(|e| e.to_string())?;

    let words = mnemonic
        .words()
        .map(|w| w.to_string())
        .collect::<Vec<String>>();
    Ok(words)
}

pub fn compute_new_secrets(old_secret: &Vec<u8>) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(None, old_secret);

    let mut okm = [0u8; 64]; // 64 bytes output
    hk.expand(b"ratchet step", &mut okm)
        .expect("HKDF expand failed");

    let (new_root_secret, message_key) = okm.split_at(32);

    (
        new_root_secret.try_into().unwrap(),
        message_key.try_into().unwrap(),
    )
}

pub async fn ratchet_forward(s_type: &str, chat_id: &str) -> Result<[u8; 32], String> {
    let current_secret = crate::database::private_chat::get_secret(s_type, chat_id).await?;
    let (new_root_secret, message_key) = compute_new_secrets(&current_secret);
    crate::database::private_chat::set_new_secret(s_type, chat_id, new_root_secret.to_vec())
        .await?;
    Ok(message_key)
}
