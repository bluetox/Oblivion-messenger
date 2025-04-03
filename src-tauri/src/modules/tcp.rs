use super::handle;
use bytes::{Buf, BytesMut};
use ring::signature::KeyPair;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tauri::command]
pub async fn send_message(dst_id_hexs: String, message_string: String) {
    let keys_lock = super::super::KEYS.lock().await;
    let keys = keys_lock.as_ref().expect("Keys not initialized");

    let dst_id_bytes = hex::decode(&dst_id_hexs).unwrap();

    let shared_secret = {
        let shared_secret_locked = super::super::SHARED_SECRETS.lock().await;
        shared_secret_locked.get(&dst_id_hexs).unwrap().clone()
    };

    let message = super::encryption::encrypt_message(&message_string, &shared_secret).await;

    let dilithium_public_key = keys.dilithium_keys.public.clone();
    let ed25519_public_key = keys.ed25519_keys.public_key().as_ref().to_vec();

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
            + keys.nonce.len()
            + timestamp_bytes.len()
            + message.len(),
    );
    sign_part.extend_from_slice(&dilithium_public_key);
    sign_part.extend_from_slice(&ed25519_public_key);
    sign_part.extend_from_slice(&dst_id_bytes);
    sign_part.extend_from_slice(&keys.nonce);
    sign_part.extend_from_slice(&timestamp_bytes);
    sign_part.extend_from_slice(&message);

    let dilithium_signature = keys.dilithium_keys.sign(&sign_part);
    let ed25519_signature = keys.ed25519_keys.sign(&sign_part).as_ref().to_vec();

    let mut raw_packet = BytesMut::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len(),
    );

    raw_packet.extend_from_slice(&[0x04, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let locked_shared_secret = super::super::NODE_SHARED_SECRET.lock().await;
    let encrypted_packet = super::encryption::encrypt_packet(&raw_packet, &locked_shared_secret).await;
    drop(locked_shared_secret);

    {
        let mut global_write_half = super::super::GLOBAL_WRITE_HALF.lock().await;
        let write_half = global_write_half.as_mut().unwrap();
        let _ = write_half.write_all(&encrypted_packet).await;
    }
}

pub async fn create_get_nodes_packet() -> Vec<u8>{
    let keys_lock = super::super::KEYS.lock().await;
    let keys = keys_lock.as_ref().ok_or("Keys not initialized").unwrap();
    
    let dilithium_public_key = &keys.dilithium_keys.public;
    let ed25519_public_key = keys.ed25519_keys.public_key().as_ref();

    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {:?}", e)).unwrap();
    let timestamp = duration_since_epoch.as_secs().to_le_bytes();

    let mut sign_part = BytesMut::with_capacity(
        dilithium_public_key.len() + ed25519_public_key.len() + keys.nonce.len() + timestamp.len()
    );
    sign_part.extend_from_slice(dilithium_public_key);
    sign_part.extend_from_slice(ed25519_public_key);
    sign_part.extend_from_slice(&keys.nonce);
    sign_part.extend_from_slice(&timestamp);

    let dilithium_signature = keys.dilithium_keys.sign(&sign_part);
    let ed25519_signature = keys.ed25519_keys.sign(&sign_part).as_ref().to_vec();

    drop(keys_lock);

    let mut raw_packet = BytesMut::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len()
    );
    raw_packet.extend_from_slice(&[0x0a, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let total_size = raw_packet.len() as u16;
    raw_packet[1..3].copy_from_slice(&total_size.to_le_bytes());
    raw_packet.to_vec()
}

pub async fn establish_ss_with_node(read_half: &mut tokio::io::ReadHalf<TcpStream>, write_half:  &mut tokio::io::WriteHalf<TcpStream>) {
    let mut message = BytesMut::with_capacity(1573);
    let total_size =  1573 as u16;
    let mut rng =   rand::rngs::OsRng;
    let keypair = pqc_kyber::Keypair::generate(&mut rng).unwrap();

    message.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00]);
    message[1..3].copy_from_slice(&total_size.to_le_bytes());
    message.extend_from_slice(&keypair.public);

    let _ = write_half.write_all(&message).await;

    let mut chunk = vec![0u8; 2048];
    let _response = read_half.read(&mut chunk).await;

    let ct = &chunk[5 .. 5 + 1568];

    let mut locked_shared_secret = super::super::NODE_SHARED_SECRET.lock().await;
    *locked_shared_secret = pqc_kyber::decapsulate(ct, &keypair.secret).unwrap().to_vec();
    println!("shared secret: {:?}", locked_shared_secret);
}

pub async fn create_server_connect_packet() -> Result<Vec<u8>, String> {
    let keys_lock = super::super::KEYS.lock().await;
    let keys = keys_lock.as_ref().ok_or("Keys not initialized")?;
    
    let dilithium_public_key = &keys.dilithium_keys.public;
    let ed25519_public_key = keys.ed25519_keys.public_key().as_ref();

    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Time error: {:?}", e))?;
    let timestamp = duration_since_epoch.as_secs().to_le_bytes();

    let mut sign_part = BytesMut::with_capacity(
        dilithium_public_key.len() + ed25519_public_key.len() + keys.nonce.len() + timestamp.len()
    );
    sign_part.extend_from_slice(dilithium_public_key);
    sign_part.extend_from_slice(ed25519_public_key);
    sign_part.extend_from_slice(&keys.nonce);
    sign_part.extend_from_slice(&timestamp);

    let dilithium_signature = keys.dilithium_keys.sign(&sign_part);
    let ed25519_signature = keys.ed25519_keys.sign(&sign_part).as_ref().to_vec();

    drop(keys_lock);

    let mut raw_packet = BytesMut::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len()
    );
    raw_packet.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let locked_shared_secret = super::super::NODE_SHARED_SECRET.lock().await;
    let encrypted_packet = super::encryption::encrypt_packet(&raw_packet, &locked_shared_secret).await;
    drop(locked_shared_secret);
    
    Ok(encrypted_packet)
}


pub async fn server_connect(app: &AppHandle) -> io::Result<()> {

    let mut stream = TcpStream::connect("192.168.1.51:8081").await?;
    let (mut read_half, mut write_half) = tokio::io::split(stream);
    let get_nodes_packet = create_get_nodes_packet().await;
    write_half.write_all(&get_nodes_packet).await?;

    let mut chunk = vec![0u8; 2048];
    let bytes_read = read_half.read(&mut chunk).await.unwrap();

    write_half.shutdown().await.expect("Failed to shut down write side");
    drop(read_half);
    drop(write_half);

    chunk.truncate(bytes_read);

    let buffer_str = match std::str::from_utf8(&chunk) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    
    let ips: Vec<String> = buffer_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    println!("received assigned nodes: {:?}", ips);

    

    let mut successful_connection = None;

    for ip in ips {
        match TcpStream::connect(format!("{}:8081", ip)).await {
            Ok(new_stream) => {
                println!("Connected to node: {}", ip);
                stream = new_stream;
                successful_connection = Some(stream);
                break;
            }
            Err(e) => {
                println!("Failed to connect to {}: {}", ip, e);
            }
        }
    }

    if let Some(stream) = successful_connection {
        let (mut read_half, mut write_half) = tokio::io::split(stream);
        establish_ss_with_node(&mut read_half, &mut write_half).await;
        let buffer = create_server_connect_packet().await.unwrap();
        write_half.write_all(&buffer).await?;

        {
            let mut global_write_half = super::super::GLOBAL_WRITE_HALF.lock().await;

            if let Some(mut existing_write_half) = global_write_half.take() {
                let _ = existing_write_half.shutdown().await;
            }

            *global_write_half = Some(write_half);
        }

        listen(&mut read_half, app).await?;
    } else {
        println!("Failed to connect to any nodes.");
    }

    Ok(())
}

pub async fn send_cyphertext(dst_id_bytes: Vec<u8>, cyphertext: Vec<u8>) -> Vec<u8> {
    let keys_lock = super::super::KEYS.lock().await;
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

    let mut raw_packet = BytesMut::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len(),
    );

    raw_packet.extend_from_slice(&[0x03, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let locked_shared_secret = super::super::NODE_SHARED_SECRET.lock().await;
    let encrypted_packet = super::encryption::encrypt_packet(&raw_packet, &locked_shared_secret).await;
    drop(locked_shared_secret);

    return encrypted_packet;
}

pub async fn send_kyber_key(dst_id_bytes: Vec<u8>) {
    let keys_lock = super::super::KEYS.lock().await;
    let keys = keys_lock.as_ref().expect("Keys not initialized");
    let kyber_public_key = keys.kyber_keys.public;
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
            + kyber_public_key.len(),
    );
    sign_part.extend_from_slice(&dilithium_public_key);
    sign_part.extend_from_slice(&ed25519_public_key);
    sign_part.extend_from_slice(&dst_id_bytes);
    sign_part.extend_from_slice(&nonce);
    sign_part.extend_from_slice(&timestamp_bytes);
    sign_part.extend_from_slice(&kyber_public_key);

    let dilithium_signature = keys.dilithium_keys.sign(&sign_part);
    let ed25519_signature = keys.ed25519_keys.sign(&sign_part).as_ref().to_vec();
    drop(keys_lock);

    let mut raw_packet = BytesMut::with_capacity(
        5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len(),
    );

    raw_packet.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00]);
    raw_packet.extend_from_slice(&dilithium_signature);
    raw_packet.extend_from_slice(&ed25519_signature);
    raw_packet.extend_from_slice(&sign_part);

    let locked_shared_secret = super::super::NODE_SHARED_SECRET.lock().await;
    let encrypted_packet = super::encryption::encrypt_packet(&raw_packet, &locked_shared_secret).await;
    drop(locked_shared_secret);

    {
        let mut global_write_half = super::super::GLOBAL_WRITE_HALF.lock().await;
        let write_half =  global_write_half.as_mut().unwrap();
        let _ = write_half.write_all(&encrypted_packet).await;
    }
}

async fn listen(
    read_half: &mut tokio::io::ReadHalf<tokio::net::TcpStream>,
    app: &AppHandle,
) -> io::Result<()> {
    let mut buffer = BytesMut::with_capacity(1024);
    let mut chunk = vec![0u8; 1024];
    loop {
        match read_half.read(&mut chunk).await {
            Ok(0) => {
                println!("Disconnected from server.");
                break;
            }
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);

                if buffer.len() < 3 {
                    println!("[ERROR] Invalid packet: too short");
                    buffer.clear();
                    continue;
                }

                let prefix = buffer[0];
                let payload_size_bytes = &buffer[1..3];
                let payload_size =
                    u16::from_le_bytes([payload_size_bytes[0], payload_size_bytes[1]]) as usize;

                if buffer.len() < payload_size {
                    continue;
                }

                match prefix {
                    2 => {
                        let response = super::handle::handle_kyber(&buffer[..payload_size].to_vec()).await.unwrap();
                        {
                            let mut global_write_half = super::super::GLOBAL_WRITE_HALF.lock().await;
                            let write_half = global_write_half.as_mut().unwrap();
                            write_half.write_all(&response).await?;
                        }
                    }
                    3 => {
                        if let Err(err) = handle::handle_ct(&buffer[..payload_size].to_vec()).await {
                            println!("Error handling ciphertext: {}", err);
                        }                        
                    }
                    4 => {
                        let _ = handle::handle_message(&buffer[..payload_size].to_vec(), app).await;
                    }
                    _ => {
                        println!("[ERROR] Invalid packet: unknown prefix {}", prefix);
                    }
                }

                buffer.advance(payload_size);
                
            }
            Err(e) => {
                eprintln!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn establish_ss(dst_user_id: String) {
    super::tcp::send_kyber_key(hex::decode(dst_user_id).unwrap()).await;
}
