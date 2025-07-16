use bytes::BytesMut;
use futures::lock;
use pq_tls::client::PqTlsClient;
use pq_tls::kem_obj::MlKem512Keypair;
use pq_tls::objects::{CAKAKeys, CSigningKeys, FrodoKem1344Keypair, PqAKAKeys, PqSigningKeys, PqTlsSettings};
use pq_tls::sign_obj::FalconPadded1024Keypair;
use ring::signature::KeyPair;
use std::path::{self, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use std::error::Error;

use crate::network::packet;

const NODE_ADDRESS: &str = "148.113.191.144";
const NODE_PORT: u16 = 32775;

pub struct TcpClient {
    stop_flag: Arc<AtomicBool>,
    pq_tls_client: Arc<Mutex<PqTlsClient>>,
    _listener: tokio::task::JoinHandle<()>,
}

impl TcpClient {
    pub async fn connect(path: PathBuf) -> Result<Self, Box<dyn Error>>{
        
        let mut settings = pq_tls::objects::PqTlsSettings{
            pq_signing_keys: PqSigningKeys::FalconPadded1024(FalconPadded1024Keypair::generate()),
            c_signing_keys: CSigningKeys::default(),
            pq_aka_keys: PqAKAKeys::MlKem512(MlKem512Keypair::generate()),
            c_aka_keys: CAKAKeys::default()
        };

        let mut client = pq_tls::client::PqTlsClient::connect(NODE_ADDRESS, NODE_PORT, &mut settings, &path).await?;

        let get_node_packet = super::packet::create_get_nodes_packet().await;
        
        client.write(&get_node_packet).await?;
        println!("wrote");
        let packet = match client.rx.lock().await.recv().await {
            Some(packet) => packet,
            None => return Err("Failed to receive packet from main node".into()),
        };
        println!("got packet");
        let buffer_str = std::str::from_utf8(&packet).map_err(|_| "Invalid UTF-8 received from main node")?;

        let ips: Vec<String> = buffer_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        println!("ips: {:?}", ips);
        
        let mut client = connect_to_first_successful(&ips, &mut settings, path).await?;
        let conn_packet = packet::create_server_connect_packet().await?;
        client.write(&conn_packet).await.unwrap();
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        let arc_client = Arc::new(Mutex::new(client));
        let mut client_clone = Arc:: clone(&arc_client);
        let handle = tokio::spawn(async move {
            if let Err(e) = super::utils::listen(client_clone, flag_clone).await {
                eprintln!("Listener error: {:?}", e);
            }
        });

        Ok(Self { stop_flag: flag, pq_tls_client: arc_client , _listener: handle})
    }

    pub async fn send_message(&mut self, chat_id: &str, dst_id_hexs: &str, message_string: &str) {
        {
            println!("sending a message");
            let ss = crate::crypto::keys::ratchet_forward(&"send_root_secret", &chat_id)
                .await
                .unwrap();
            println!("{:?}", ss);
            let packet = crate::network::packet::create_send_message_packet(
                dst_id_hexs,
                message_string,
                &ss.to_vec()
            )
            .await
            .unwrap();
            {
                let mut locked = self.pq_tls_client.lock().await;
                locked.write(&packet).await.unwrap();
            }
        }
    }

    pub async fn send_kyber_key(
        &mut self,
        dst_id_bytes: Vec<u8>,
        kyber_keys: &safe_pqc_kyber::Keypair,
    ) {
        let keys_lock = crate::GLOBAL_KEYS.lock().await;
        let keys = keys_lock.as_ref().expect("Keys not initialized");

        let kyber_public_key = kyber_keys.public;

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

        let mut packet = Vec::with_capacity(
            5 + dilithium_signature.len() + ed25519_signature.len() + sign_part.len(),
        );

        packet.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00]);
        packet.extend_from_slice(&dilithium_signature);
        packet.extend_from_slice(&ed25519_signature);
        packet.extend_from_slice(&sign_part);
        {

            let mut locked = self.pq_tls_client.lock().await;
            locked.write(&packet).await.unwrap();
        }
    }

    pub async fn write(&mut self, data: &Vec<u8>) {
        
        let mut locked = self.pq_tls_client.lock().await;
        locked.write(data).await.unwrap();
    }

    pub async fn shutdown(&mut self) -> Result<(), String> {
        self.stop_flag.store(true, Ordering::Relaxed);


        Ok(())
    }
}


pub async fn connect_to_first_successful(ips: &[String], settings: &mut PqTlsSettings, path: PathBuf) 
    -> Result<PqTlsClient, Box<dyn std::error::Error>> 
{
    for ip in ips {
        match pq_tls::client::PqTlsClient::connect(ip, NODE_PORT, settings, &path).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", ip, e);
                continue;
            }
        }
    }

    Err("Failed to connect to any node".into())
}