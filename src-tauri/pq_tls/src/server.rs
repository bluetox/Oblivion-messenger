use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use std::error::Error;

use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;


use crate::objects::*;
use crate::{settings, crypto, utils};

pub struct PqTlsServer {
    reader: ReadHalf<TcpStream>,
    writer: WriteHalf<TcpStream>,
    ss: [u8; 32],
    buffer: Vec<u8>
}

impl PqTlsServer {
    pub async fn handle_new_client(stream: TcpStream) -> Result<Self, Box<dyn Error>> {
        let (mut reader, mut writer) = tokio::io::split(stream);
        
        let mut client_hello = [0u8; 1024];

        let mut buffer = Vec::new();

        let (settings_bytes, client_random) = match reader.read(&mut client_hello).await {
            Ok(n) if n == 0 => {
                return Err("Connection terminated".into());
            }
            Ok(n) => {
                if n != 41 {
                    return Err("Invalid size for client hello".into());
                }
                (&client_hello[1..9], &client_hello[9..41])
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        let mut settings = settings::bytes_to_settings(settings_bytes);

        let pq_sign_pk_size = settings.pq_signing_keys.pk_size();
        let pq_sign_size = settings.pq_signing_keys.sign_size();
        let c_sign_pk_size = settings.c_signing_keys.pk_size();
        let c_sign_size = settings.c_signing_keys.sign_size();
        let pq_aka_ct_size = settings.pq_aka_keys.ct_size();
        let c_aka_pk_size = settings.c_aka_keys.pk_size();

        let server_random = server_hello(&mut settings, &mut writer).await?;

        let client_ct = get_client_ct(&mut reader, &settings, &mut buffer).await?;

 
        let pq_ct = &client_ct[1 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size .. 1 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_ct_size];
        let c_pk_client = &client_ct[1 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_ct_size .. 1 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size + pq_aka_ct_size + c_aka_pk_size];
        let pq_sign_pk = &client_ct[1 + pq_sign_size + c_sign_size .. 1 + pq_sign_size + c_sign_size + pq_sign_pk_size];
        let c_sign_pk = &client_ct[1 + pq_sign_size + c_sign_size + pq_sign_pk_size .. 1 + pq_sign_size + c_sign_size + pq_sign_pk_size + c_sign_pk_size];
        
        let pq_sign = &client_ct[1 .. 1 + pq_sign_size];
        let c_sign = &client_ct[1 + pq_sign_size .. 1 + pq_sign_size + c_sign_size];

        let signed_part = &client_ct[1 + pq_sign_size + c_sign_size ..];

        settings.pq_signing_keys.verify(signed_part, pq_sign_pk, pq_sign)?;
        settings.c_signing_keys.verify(signed_part, c_sign_pk, c_sign)?;

        let pubkey_array: &[u8; 32] = c_sign_pk.try_into().expect("slice with incorrect length");
        let verifying_key: VerifyingKey = VerifyingKey::from_bytes(pubkey_array)?;
        let signature: Signature = Signature::try_from(&c_sign[..])?;
        verifying_key.verify_strict(signed_part, &signature)?;

        
        let pq_ss = settings.pq_aka_keys.decapsulate(pq_ct);
        let c_ss = settings.c_aka_keys.decapsulate(c_pk_client);

        let ss = crypto::derive_hybrid_key(client_random, &server_random, &pq_ss, &c_ss);

        let ss_array: [u8; 32] = ss.try_into().expect("slice with incorrect length");
        Ok(
            Self { 
                reader: reader, 
                writer: writer,
                ss: ss_array,
                buffer: buffer
            }
        )
    }


    pub async fn wait_for_packet(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        loop {
            while self.buffer.len() >= 4 {

                let size = (self.buffer[1] as usize)

                    | ((self.buffer[2] as usize) << 8)

                    | ((self.buffer[3] as usize) << 16);

                if self.buffer.len() < size {
                    break;

                }
                let packet = self.buffer[..size].to_vec();
                self.buffer.drain(0..size);

                let dec_packet = crypto::decrypt_packet(&packet, &self.ss);

                return Ok(dec_packet);
            }

            let mut chunk = [0u8; 1024];

            let n = match self.reader.read(&mut chunk).await {

                Ok(0) => return Err("Connection terminated".into()),
                Ok(n) => n,
                Err(e) => return Err(format!("Error reading from stream: {}", e).into()),

            };

            self.buffer.extend_from_slice(&chunk[..n]);

        }

    }
    pub async fn send_dummy_packet(&mut self) -> Result<(), std::io::Error> {
        let payload = b"Hello from dummy packet!";

        let state: u8 = 2;

        let size = 1 + 3 + payload.len();

        let mut header = [0u8; 3];
        header[0] = (size & 0xFF) as u8;
        header[1] = ((size >> 8) & 0xFF) as u8;
        header[2] = ((size >> 16) & 0xFF) as u8;

        let mut packet = Vec::with_capacity(size);
        packet.push(state);
        packet.extend_from_slice(&header);
        packet.extend_from_slice(payload);


        let enc_packet = crypto::encrypt_packet(&packet, &self.ss);
        self.writer.write_all(&enc_packet).await?;
        Ok(())
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        
        let header: [u8; 4] = [2,0,0,0];

        let mut packet : Vec<u8> = Vec::new();
        packet.extend_from_slice(&header);
        packet.extend_from_slice(data);

        let enc_packet = crypto::encrypt_packet(&packet, &self.ss);

        self.writer.write(&enc_packet).await?;
        Ok(())
    }
}


pub async fn server_start() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:1234").await?;
    println!("Server listening on 127.0.0.1:1234");

    loop {
        let (stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            let mut res_s = match PqTlsServer::handle_new_client(stream).await {
                Ok(c) => c,
                Err(_) => {
                    return;
                }
            };

            loop {
                let packet = match res_s.wait_for_packet().await {
                    Ok(packet) => packet,
                    Err(_) => return
                };
                let _msg = String::from_utf8(packet).unwrap();
                res_s.send_dummy_packet().await.unwrap();
            }
            

        });
    }
}

pub async fn get_client_ct(
    reader: &mut ReadHalf<tokio::net::TcpStream>,
    settings: &PqTlsSettings,
    buffer: &mut Vec<u8>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let pq_sign_pk_size = settings.pq_signing_keys.pk_size();
    let pq_sign_size = settings.pq_signing_keys.sign_size();
    let c_sign_pk_size = settings.c_signing_keys.pk_size();
    let c_sign_size = settings.c_signing_keys.sign_size();
    let pq_aka_ct_size = settings.pq_aka_keys.ct_size();
    let c_aka_pk_size = settings.c_aka_keys.pk_size();

    let total_expected_packet_size = 1
        + pq_sign_pk_size
        + pq_sign_size
        + c_sign_pk_size
        + c_sign_size
        + pq_aka_ct_size
        + c_aka_pk_size;

    let mut client_ct = Vec::new();

    while client_ct.len() < total_expected_packet_size {
        let mut chunk = [0u8; 1024];
        let n = match reader.read(&mut chunk).await {
            Ok(0) => return Err("Connection terminated".into()),
            Ok(n) => n,
            Err(e) => return Err(e.into()),
        };

        let needed = total_expected_packet_size - client_ct.len();
        if n <= needed {
            client_ct.extend_from_slice(&chunk[..n]);
        } else {
            client_ct.extend_from_slice(&chunk[..needed]);
            buffer.extend_from_slice(&chunk[needed..n]);
        }
    }

    Ok(client_ct)
}


async fn server_hello(settings: &mut PqTlsSettings, writer: &mut WriteHalf<TcpStream>) -> Result<[u8; 32], Box<dyn Error>>  {
    let server_random = utils::generate_server_random();

    let pq_sign_pk = settings.pq_signing_keys.public();
    let c_sign_pk = settings.c_signing_keys.public();
    let pq_aka_pk = settings.pq_aka_keys.public();
    let c_aka_pk = settings.c_aka_keys.public();

    let mut signed_part: Vec<u8> = Vec::new(); 
    signed_part.extend_from_slice(&pq_sign_pk);
    signed_part.extend_from_slice(&c_sign_pk);
    signed_part.extend_from_slice(&pq_aka_pk);
    signed_part.extend_from_slice(&c_aka_pk);

    let pq_sign = settings.pq_signing_keys.sign(&signed_part);
    let c_sign = settings.c_signing_keys.sign(&signed_part);

    let mut server_hello: Vec<u8> = Vec::new();
    server_hello.push(1);
    server_hello.extend_from_slice(&server_random);
    server_hello.extend_from_slice(&pq_sign);
    server_hello.extend_from_slice(&c_sign);
    server_hello.extend_from_slice(&signed_part);

    writer.write(&server_hello).await?;
    Ok(server_random)
}
