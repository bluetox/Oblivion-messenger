use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt, WriteHalf, ReadHalf};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::error::Error;
use tokio::sync::Mutex;
use std::path::PathBuf;
use crate::{constants::*, objects::*, utils, settings, crypto};

pub struct PqTlsClient {
    pub writer: Arc<Mutex<WriteHalf<TcpStream>>>,
    pub rx: Arc<Mutex<Receiver<Vec<u8>>>>, // ✅ pour pouvoir faire .lock().await
    pub client_random: [u8; 32],
    pub ss: [u8; 32],
}

impl PqTlsClient {
    pub async fn connect(ip: &str, port: u16, settings: &mut PqTlsSettings, path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let addr = format!("{}:{}", ip, port);
        let db_path = path.join("known_hosts").join("db.sqlite");
        let db = crate::database::setup_db(&db_path).await;
        let stream = TcpStream::connect(addr).await?;
        let (mut reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(Mutex::new(writer));
        let client_random = utils::generate_client_random();

        // Phase 1: Client Hello
        let settings_bytes = settings::settings_to_bytes(&settings);
        let mut client_hello = vec![CLIENT_HELLO_CODE];
        client_hello.extend_from_slice(&settings_bytes);
        client_hello.extend_from_slice(&client_random);
        writer.lock().await.write_all(&client_hello).await?;

        // Phase 2: Read Server Hello
        let server_hello = Self::get_server_hello(&mut reader, settings).await?;

        
        let server_random = &server_hello[1 .. 1 + 32];

        let pq_sign_pk_size = settings.pq_signing_keys.pk_size();
        let pq_sign_size = settings.pq_signing_keys.sign_size();
        let c_sign_pk_size = settings.c_signing_keys.pk_size();
        let c_sign_size = settings.c_signing_keys.sign_size();
        let pq_aka_pk_size = settings.pq_aka_keys.pk_size();
        let c_aka_pk_size = settings.c_aka_keys.pk_size();

        let pq_sign = &server_hello[1 + 32 .. 1 + 32 + pq_sign_size];
        let c_sign = &server_hello[1 + 32 + pq_sign_size .. 1 + 32 + pq_sign_size + c_sign_size];
        let pq_sign_pk = &server_hello[1 + 32 + pq_sign_size + c_sign_size .. 1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size];
        let c_sign_pk = &server_hello[1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size .. 1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size];
        let pq_kem_pk = &server_hello[1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size .. 1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_pk_size];
        let c_aka_pk = &server_hello[1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_pk_size .. 1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_pk_size + c_aka_pk_size];

        let signed_part = &server_hello[1 + 32 + pq_sign_size + c_sign_size .. ];

        settings.pq_signing_keys.verify(signed_part, pq_sign_pk, pq_sign)?;
        settings.c_signing_keys.verify(signed_part, c_sign_pk, c_sign)?;

        let fingerprint = utils::hash_combined(pq_sign_pk, c_sign_pk);

        let known = crate::database::fingerprint_exists(&db, &fingerprint).await?;

        if !known {
            
            match crate::database::insert_new_peer(&db, pq_sign_pk, &settings.pq_signing_keys.key_type(), c_sign_pk, &settings.c_signing_keys.key_type() , &fingerprint, ip, port).await {
                Ok(()) => {},
                Err(_e) => {
                    //return Err(format!("Invalid fingerprint for host: {} {}", ip, e).into())
                }
            }
        }

        let (pq_ct, pq_ss) = settings.pq_aka_keys.encapsulate(pq_kem_pk);
        let (c_aka_p_pk, c_ss) = crypto::decapsulate_x25519(c_aka_pk);
        let ss = crypto::derive_hybrid_key(&client_random, &server_random, &pq_ss, &c_ss);
        let ss_slice = ss.try_into().unwrap();

        // Phase 3: Envoi client_ct
        let mut signed_part = Vec::new();
        signed_part.extend_from_slice(&settings.pq_signing_keys.public());
        signed_part.extend_from_slice(&settings.c_signing_keys.public());
        signed_part.extend_from_slice(&pq_ct);
        signed_part.extend_from_slice(&c_aka_p_pk);
        let pq_sign = settings.pq_signing_keys.sign(&signed_part);
        let c_sign = settings.c_signing_keys.sign(&signed_part);
        let mut client_ct = vec![2];
        client_ct.extend_from_slice(&pq_sign);
        client_ct.extend_from_slice(&c_sign);
        client_ct.extend_from_slice(&signed_part);
        writer.lock().await.write_all(&client_ct).await?;

        // Start background reader task
        let (tx, rx) = channel::<Vec<u8>>(32);
        tokio::spawn(Self::reader_loop(reader, ss_slice, tx));

        Ok(Self {
            writer,
            rx: Arc::new(Mutex::new(rx)),
            client_random,
            ss: ss_slice,
        })
    }

    async fn reader_loop(
        mut reader: tokio::io::ReadHalf<TcpStream>,
        ss: [u8; 32],
        tx: Sender<Vec<u8>>,
    ) {
        let mut buffer = Vec::new();
        loop {
            let mut chunk = [0u8; 1024];
            match reader.read(&mut chunk).await {
                Ok(0) => break, // EOF
                Ok(n) => buffer.extend_from_slice(&chunk[..n]),
                Err(_) => break,
            }

            while buffer.len() >= 4 {
                let size = (buffer[1] as usize)
                    | ((buffer[2] as usize) << 8)
                    | ((buffer[3] as usize) << 16);

                if buffer.len() < size {
                    break;
                }

                let packet = buffer[..size].to_vec();
                buffer.drain(0..size);

                let dec_packet = crypto::decrypt_packet(&packet, &ss);
                let _ = tx.send(dec_packet).await;
            }
        }
    }

    async fn get_server_hello(reader: &mut ReadHalf<TcpStream>, settings: &PqTlsSettings) -> Result<Vec<u8>, Box<dyn Error>> {
        let pq_sign_pk_size = settings.pq_signing_keys.pk_size();
        let pq_sign_size = settings.pq_signing_keys.sign_size();
        let c_sign_pk_size = settings.c_signing_keys.pk_size();
        let c_sign_size = settings.c_signing_keys.sign_size();
        let pq_aka_pk_size = settings.pq_aka_keys.pk_size();
        let c_aka_pk_size = settings.c_aka_keys.pk_size();

        let mut server_hello = Vec::new();
        while server_hello.len() < 1 + 32 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_pk_size + c_aka_pk_size{
            let mut chunk = [0u8; 1024];
            match reader.read(&mut chunk).await {
                Ok(n) if n == 0 => {
                    return Err("Connection terminated".into());
                }
                Ok(n) => {
                    server_hello.extend_from_slice(&chunk[..n]);

                }
                Err(e) => {
                    return Err(e.into());
                }
            };
        }
        Ok(server_hello)
    }
    pub async fn write(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let mut packet = vec![2, 0, 0, 0];
        packet.extend_from_slice(data);
        let enc_packet = crypto::encrypt_packet(&packet, &self.ss);
        self.writer.lock().await.write_all(&enc_packet).await?;
        Ok(())
    }

    pub async fn pull_packet(&mut self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }
}
