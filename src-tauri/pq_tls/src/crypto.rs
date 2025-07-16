use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use rand::{rngs::OsRng, RngCore};
use hkdf::Hkdf;
use sha2::Sha256;
use chacha20::ChaCha20;
use cipher::{KeyIvInit, StreamCipher};

pub fn decapsulate_x25519(pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let arr: [u8; 32] = pk.try_into().expect("Failed to convert slice to array");

    let public_key = X25519PublicKey::from(arr);

    let secret = EphemeralSecret::random_from_rng(OsRng);

    let my_public_key = X25519PublicKey::from(&secret);
    let shared_secret = secret.diffie_hellman(&public_key);

    (my_public_key.as_bytes().to_vec(), shared_secret.as_ref().to_vec())
}

pub fn derive_hybrid_key(client_random: &[u8], server_random: &[u8], ss_pq: &[u8], ss_classical: &[u8]) -> Vec<u8> {
    let mut combined_ss = Vec::with_capacity(ss_pq.len() + ss_classical.len());

    combined_ss.extend_from_slice(ss_pq);
    combined_ss.extend_from_slice(ss_classical);

    let mut salt = Vec::new();
    salt.extend_from_slice(client_random);
    salt.extend_from_slice(server_random);

    let hk = Hkdf::<Sha256>::new(Some(&salt), &combined_ss);

    let info = b"pq-classical hybrid handshake key";
    let mut okm = [0u8; 32];
    hk.expand(info, &mut okm).expect("HKDF expand failed");

    okm.to_vec()
}


pub fn encrypt_packet(packet: &Vec<u8>, key: &[u8; 32]) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    let payload = &packet[4..];

    let mut ciphertext = payload.to_vec();
    let mut encryptor = ChaCha20::new(key.into(), &nonce.into());
    encryptor.apply_keystream(&mut ciphertext);

    let size = 4 + nonce.len() + ciphertext.len();

    let mut header = packet[0..4].to_vec();
    header[1] = (size & 0xFF) as u8;
    header[2] = ((size >> 8) & 0xFF) as u8;
    header[3] = ((size >> 16) & 0xFF) as u8;

    let mut enc_packet: Vec<u8> = Vec::with_capacity(size);
    enc_packet.extend_from_slice(&header);
    enc_packet.extend_from_slice(&nonce);
    enc_packet.extend_from_slice(&ciphertext);

    enc_packet
}

pub fn decrypt_packet(enc_packet: &Vec<u8>, key: &[u8; 32]) -> Vec<u8> {
    let nonce = &enc_packet[4..16];
    let ciphertext = &enc_packet[16..];

    let mut decrypted_text = ciphertext.to_vec();
    let mut decryptor = ChaCha20::new(key.into(), nonce.into());
    decryptor.apply_keystream(&mut decrypted_text);

    decrypted_text
}
