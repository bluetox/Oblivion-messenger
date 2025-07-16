use crate::groups::tcp_handles;
use blake3::Hasher;
use bytes::{Buf, BytesMut};
use pq_tls::client::PqTlsClient;
use ring::signature::KeyPair;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tokio::io;

pub async fn send_cyphertext(dst_id_bytes: Vec<u8>, cyphertext: Vec<u8>) {
    let keys_lock = crate::GLOBAL_KEYS.lock().await;
    let keys = keys_lock.as_ref().expect("Keys not initialized");

    let dilithium_public_key = keys.dilithium_keys.public.clone();
    let ed25519_public_key = keys.ed25519_keys.public_key().as_ref().to_vec();
    let nonce = keys.nonce;
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let timestamp = duration_since_epoch.as_secs() as u64;
    let timestamp_bytes = timestamp.to_le_bytes();

    let mut sign_part = BytesMut::with_capacity(
        dilithium_public_key.len()
            + ed25519_public_key.len()
            + dst_id_bytes.len()
            + nonce.len()
            + timestamp_bytes.len()
            + cyphertext.len(),
    );
    sign_part.extend_from_slice(&dilithium_public_key);
    sign_part.extend_from_slice(&ed25519_public_key);
    sign_part.extend_from_slice(&dst_id_bytes);
    sign_part.extend_from_slice(&nonce);
    sign_part.extend_from_slice(&timestamp_bytes);
    sign_part.extend_from_slice(&cyphertext);

    let dilithium_signature = keys.dilithium_keys.sign(&sign_part);
    let ed25519_signature = keys.ed25519_keys.sign(&sign_part).as_ref().to_vec();
    drop(keys_lock);

    let mut raw_packet = Vec::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len(),
    );

    raw_packet.extend_from_slice(&[0x03, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let mut tcp_guard = crate::GLOBAL_CLIENT.lock().await;

    if let Some(tcp_client) = tcp_guard.as_mut() {
        tcp_client.write(&raw_packet).await;
    } else {
        println!("No existing TCP client found");
    }
}

pub async fn listen(
    client: Arc<Mutex<PqTlsClient>>,
    flag: Arc<AtomicBool>,
) -> io::Result<()> {

    let rx_arc = {
        let guard = client.lock().await;
        guard.rx.clone()
    };

    let mut rx = rx_arc.lock().await;

    while !flag.load(Ordering::Relaxed) {
        let packet = match rx.recv().await {
            Some(pkt) => pkt,
            None => {
                println!("Channel closed, exiting...");
                break;
            }
        };
        let prefix = packet[0]; 
                match prefix {
                    0x02 => {
                        println!("received kyber_key");
                        match crate::network::handle::handle_kyber(&packet)
                            .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Error handling kyber: {}", err);
                            }
                        }
                    }
                    0x03 => {
                        println!("received ct");
                        if let Err(err) =
                            crate::network::handle::handle_ct(&packet)
                                .await
                        {
                            println!("Error handling ciphertext: {}", err);
                        }
                    }
                    0x04 => {
                        match crate::network::handle::handle_message(
                            &packet,
                        )
                        .await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Error handling message: {}", err);
                            }
                        }
                    }
                    0xC0 => {
                        tcp_handles::handle_group_invite(&packet).await;
                        // PERFECTLY CORRECT
                    }
                    0xC1 => {
                        tcp_handles::handle_group_accept(&packet).await;
                    }
                    0xC2 => {
                        tcp_handles::handle_hello(&packet).await;
                    }
                    0xC5 => {
                        tcp_handles::handle_update(&packet).await;
                    }
                    0xC6 => {
                        // DELETE HANDLE
                    }
                    0xC7 => {
                        // HANDLE MESSAGE
                        tcp_handles::handle_message(&packet).await;
                    }
                    0xF0 => {
                        let arc_app = crate::GLOBAL_STORE.get().expect("not initialized").clone();
                        let app = arc_app.lock().await;

                        let mut hasher = Hasher::new();
                        hasher.update(&packet);
                        let _ = hasher.finalize();
                        app.emit("received-video", &packet).unwrap();
                    }
                    0xFF => {
                        println!("Connexion broken exiting....");
                        
                        break;
                    }
                    _ => {
                        println!("[ERROR] Invalid packet: unknown prefix {}", prefix);
                    }
                }
            }
            Ok(())
}