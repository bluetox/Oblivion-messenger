use std::time::{SystemTime, UNIX_EPOCH};
use rand::rngs::OsRng;
use rand::RngCore;

pub fn generate_client_random() -> [u8; 32] {
    let mut client_random = [0u8; 32];

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32;

    client_random[..4].copy_from_slice(&timestamp.to_be_bytes());

    OsRng.fill_bytes(&mut client_random[4..]);
    client_random
}

pub fn generate_server_random() -> [u8; 32] {
    let mut client_random = [0u8; 32];

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32;

    client_random[..4].copy_from_slice(&timestamp.to_be_bytes());

    OsRng.try_fill_bytes(&mut client_random[4..]).unwrap();
    client_random
}

pub fn hash_combined(input1: &[u8], input2: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(input1);
    hasher.update(input2);
    hasher.finalize().into()
}