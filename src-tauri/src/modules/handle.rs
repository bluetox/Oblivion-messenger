use super::{encryption, utils};
use rand::rngs::OsRng;
use tauri::{AppHandle, Emitter};

pub async fn handle_ct(buffer: &Vec<u8>) -> Result<(), String> {
    let dilithium_signature = &buffer[5..5 + 3293];
    let ed25519_signature = &buffer[5 + 3293..5 + 3293 + 64];

    let dilihium_pub_key = &buffer[5 + 3293 + 64..5 + 3293 + 64 + 1952];
    let ed25519_public_key = &buffer[5 + 3293 + 64 + 1952..5 + 3293 + 64 + 1952 + 32];
    let src_id_nonce = &buffer[5 + 3293 + 64 + 1952 + 32 + 32..5 + 3293 + 64 + 1952 + 32 + 32 + 16];
    let ct = &buffer[5 + 3293 + 64 + 1952 + 32 + 32 + 16 + 8..5 + 3293 + 64 + 1952 + 32 + 32 + 16 + 8 + 1568];

    let data_to_sign_bytes = &buffer[5 + 3293 + 64..];

    let full_hash_input = [
        &dilihium_pub_key[..],
        &ed25519_public_key[..],
        &src_id_nonce[..],
    ]
    .concat();

    if pqc_dilithium::verify(&dilithium_signature, &data_to_sign_bytes, &dilihium_pub_key).is_err() {
        println!("[ERROR] Invalid Dilithium signature, dropping message.");
        return Err("Invalid Dilithium signature".to_string());
    }

    let public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &ed25519_public_key);
    if public_key.verify(&data_to_sign_bytes, &ed25519_signature).is_err() {
        println!("[ERROR] Invalid Ed25519 signature, dropping message.");
        return Err("Invalid Ed25519 signature".to_string());
    }

    let dst_id_hex = utils::create_user_id_hash(&full_hash_input);

    {
        let keys_lock = super::super::KEYS.lock().await;
        let keys = keys_lock.as_ref().expect("Keys not initialized");

        let ss = pqc_kyber::decapsulate(ct, &keys.kyber_keys.secret)
            .map_err(|e| format!("Kyber decapsulation failed: {:?}", e))?;

        let mut locked_shared_secrets = super::super::SHARED_SECRETS.lock().await;
        locked_shared_secrets.insert(dst_id_hex.clone(), ss.to_vec());

        super::database::save_shared_secret(dst_id_hex.clone().as_ref(), ss.to_vec())
            .await
            .map_err(|e| format!("Failed to save shared secret: {:?}", e))?;
    }

    Ok(())
}

pub async fn handle_kyber(buffer: &Vec<u8>) -> Result<Vec<u8>, String> {
    let mut rng = OsRng;

    let dilithium_signature = &buffer[5..5 + 3293];
    let ed25519_signature = &buffer[5 + 3293..5 + 3293 + 64];

    let dilithium_pub_key = &buffer[5 + 3293 + 64..5 + 3293 + 64 + 1952];
    let ed25519_public_key = &buffer[5 + 3293 + 64 + 1952..5 + 3293 + 64 + 1952 + 32];
    let src_id_nonce = &buffer[5 + 3293 + 64 + 1952 + 32 + 32..5 + 3293 + 64 + 1952 + 32 + 32 + 16];
    let kyber_public_key =
        &buffer[5 + 3293 + 64 + 1952 + 32 + 32 + 16 + 8..5 + 3293 + 64 + 1952 + 32 + 32 + 16 + 8 + 1568];

    let data_to_sign_bytes = &buffer[5 + 3293 + 64..];

    let full_hash_input = [&dilithium_pub_key[..], &ed25519_public_key[..], &src_id_nonce[..]].concat();

    if pqc_dilithium::verify(&dilithium_signature, &data_to_sign_bytes, &dilithium_pub_key).is_err() {
        println!("[ERROR] Invalid Dilithium signature, dropping message.");
        return Err("Invalid Dilithium signature".to_string());
    }

    let public_key =  ring::signature::UnparsedPublicKey::new(& ring::signature::ED25519, &ed25519_public_key);
    if public_key.verify(&data_to_sign_bytes, &ed25519_signature).is_err() {
        println!("[ERROR] Invalid Ed25519 signature, dropping message.");
        return Err("Invalid Ed25519 signature".to_string());
    }

    let (ciphertext, shared_secret) = match pqc_kyber::encapsulate(&kyber_public_key, &mut rng) {
        Ok(result) => result,
        Err(_) => return Err("Kyber encapsulation failed".to_string()),
    };

    let dst_id_hex = utils::create_user_id_hash(&full_hash_input);

    let mut locked_shared_secrets = super::super::SHARED_SECRETS.lock().await;
    locked_shared_secrets.insert(dst_id_hex.clone(), shared_secret.to_vec());

    if super::database::save_shared_secret(dst_id_hex.clone().as_ref(), shared_secret.to_vec())
        .await
        .is_err()
    {
        return Err("Failed to save shared secret".to_string());
    }

    let dst_id_bytes = match hex::decode(dst_id_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Failed to decode dst_id_hex".to_string()),
    };
    
    let response = super::tcp::send_cyphertext(dst_id_bytes, ciphertext.to_vec()).await;

    Ok(response)
}

pub async fn handle_message(buffer: &Vec<u8>, app: &AppHandle) -> Result<(), String> {
    let dilithium_signature = &buffer[5 .. 5 + 3293];
    let ed25519_signature = &buffer[5 + 3293 .. 5 + 3293 + 64];

    let dilithium_pub_key = &buffer[5 + 3293 + 64.. 5 + 3293 + 64 + 1952];
    let ed25519_pub_key = &buffer[5 + 3293 + 64 + 1952..5 + 3293 + 64 + 1952 + 32];
    let src_id_nonce = &buffer[5 + 3293 + 64 + 1952 + 32 + 32..5 + 3293 + 64 + 1952 + 32 + 32 + 16];

    let data_to_sign_bytes = &buffer[5 + 3293 + 64 ..];

    let full_hash_input = [
        &dilithium_pub_key[..],
        &ed25519_pub_key[..],
        &src_id_nonce[..],
    ]
    .concat();

    if !pqc_dilithium::verify(&dilithium_signature, &data_to_sign_bytes, &dilithium_pub_key).is_ok() {
        println!("[ERROR] Invalid Dilithium signature, dropping message.");
        return Err("Invalid Dilithium signature".to_string());
    }

    let public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &ed25519_pub_key);
    if let Err(_) = public_key.verify(&data_to_sign_bytes, &ed25519_signature) {
        println!("[ERROR] Invalid Ed25519 signature, dropping message.");
        return Err("Invalid Ed25519 signature".to_string());
    }

    let dst_id_hex = utils::create_user_id_hash(&full_hash_input);

    let locked_shared_secrets = super::super::SHARED_SECRETS.lock().await;
    let shared_secret = match locked_shared_secrets.get(&dst_id_hex) {
        Some(secret) => secret.clone(),
        None => {
            println!("[ERROR] No shared secret found for {}", dst_id_hex);
            return Err(format!("No shared secret found for {}", dst_id_hex));
        }
    };

    match encryption::decrypt_message(
        &buffer[5 + 3293 + 64 + 1952 + 32 + 32 + 16 + 8..].to_vec(),
        &shared_secret,
    )
    .await
    {
        Ok(decrypted_message) => {
            app.emit(
                "received-message",
                format!(
                    "{{\"source\": \"{}\", \"message\": \"{}\"}}",
                    dst_id_hex, decrypted_message
                ),
            ).unwrap();
            Ok(())
        }
        Err(e) => {
            println!("[ERROR] Decryption failed: {:?}", e);
            Err(format!("Decryption failed: {:?}", e))
        }
    }
}
