use aes_gcm::AeadCore;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::rngs::OsRng;

pub async fn encrypt_message(plain_text: &str, key_buffer: &Vec<u8>) -> Vec<u8> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let key = Key::<Aes256Gcm>::from_slice(&key_buffer);

    let cipher = Aes256Gcm::new(&key);

    match cipher.encrypt(&nonce, plain_text.as_bytes()) {
        Ok(mut encrypted_data) => {
            let mut result = nonce.to_vec();
            result.append(&mut encrypted_data);
            result
        }
        Err(e) => {
            println!("Error encrypting message: {:?}", e);
            Vec::new()
        }
    }
}
pub async fn encrypt_data(data: &[u8], key_buffer: &Vec<u8>) -> Vec<u8> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let key = Key::<Aes256Gcm>::from_slice(&key_buffer);

    let cipher = Aes256Gcm::new(&key);

    match cipher.encrypt(&nonce, data) {
        Ok(mut encrypted_data) => {
            let mut result = nonce.to_vec();
            result.append(&mut encrypted_data);
            result
        }
        Err(e) => {
            println!("Error encrypting message: {:?}", e);
            Vec::new()
        }
    }
}

pub async fn decrypt_message(
    encrypted_buffer: &Vec<u8>,
    key_buffer: &Vec<u8>,
) -> Result<String, String> {
    if encrypted_buffer.len() < 12 {
        return Err("Encrypted buffer is too short".to_string());
    }

    let iv = &encrypted_buffer[0..12];
    let cipher_text_buffer = &encrypted_buffer[12..];

    let key = Key::<Aes256Gcm>::from_slice(&key_buffer);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(iv);

    match cipher.decrypt(nonce, cipher_text_buffer.as_ref()) {
        Ok(result) => match String::from_utf8(result) {
            Ok(text) => {
                let cleaned_text = text.replace("&nbsp;", " ");
                Ok(cleaned_text)
            }
            Err(e) => Err(format!("UTF-8 conversion error: {}", e)),
        },
        Err(e) => Err(format!("Decryption failed: {:?}", e)),
    }
}

pub async fn decrypt_data(
    encrypted_buffer: &Vec<u8>,
    key_buffer: &Vec<u8>,
) -> Result<Vec<u8>, String> {
    if encrypted_buffer.len() < 12 {
        return Err("Encrypted buffer is too short".to_string());
    }

    let iv = &encrypted_buffer[0..12];
    let cipher_text_buffer = &encrypted_buffer[12..];

    let key = Key::<Aes256Gcm>::from_slice(&key_buffer);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(iv);

    match cipher.decrypt(nonce, cipher_text_buffer.as_ref()) {
        Ok(result) => return Ok(result),
        Err(e) => Err(format!("Decryption failed: {:?}", e)),
    }
}
