use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey, Signature as Ed25519Signature, Signer};
use rand::rngs::OsRng;
use rand::RngCore;
use std::error::Error;
use frodo_kem::Algorithm;
use frodo_kem::{EncryptionKey as FrodoKemPub, DecryptionKey as FrodoKemPrivate};
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Write;
use std::io::Read;


use crate::kem_obj::*;
use crate::sign_obj::*;
use crate::constants::*;

pub struct FrodoKem1344Keypair {
    pub secret: FrodoKemPrivate,
    pub public: FrodoKemPub
}

pub struct FrodoKem976Keypair {
    pub secret: FrodoKemPrivate,
    pub public: FrodoKemPub
}

pub struct FrodoKem640Keypair {
    pub secret: FrodoKemPrivate,
    pub public: FrodoKemPub
}



pub struct Ed25519Keypair(Ed25519SigningKey);

pub struct X25519Keypair{
    pub secret: EphemeralSecret,
    pub public: X25519PublicKey
}

impl X25519Keypair {
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        X25519Keypair { secret, public}
    }
}

impl FrodoKem1344Keypair {
    pub fn generate() -> Self {
        let alg = Algorithm::FrodoKem1344Shake;
        let (ek, dk) = alg.generate_keypair(OsRng);
        FrodoKem1344Keypair { secret: dk, public: ek }
    }
    pub fn public_key(&self) -> Vec<u8> {
        self.public.value().to_vec()
    }

    pub fn decapsulate(&self, ct: &[u8]) -> Vec<u8> {
        let alg = Algorithm::FrodoKem1344Shake;
        let decap = alg
            .decapsulate(&self.secret, &alg.ciphertext_from_bytes(ct).unwrap())
            .unwrap();

        decap.0.value().to_vec()
    }

    pub fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let ek = Algorithm::FrodoKem1344Shake.encryption_key_from_bytes(pk).unwrap();
        let (ct, ss) = Algorithm::FrodoKem1344Shake.encapsulate_with_rng(&ek, &mut OsRng).unwrap();
        
        (ct.value().to_vec(), ss.value().to_vec())
    }

    pub fn pk_size(&self) -> usize {
        Algorithm::FrodoKem1344Shake.params().encryption_key_length
    }

    pub fn ct_size(&self) -> usize {
        Algorithm::FrodoKem1344Shake.params().ciphertext_length
    }
}

impl FrodoKem976Keypair {
    pub fn generate() -> Self {
        let alg = Algorithm::FrodoKem976Shake;
        let (ek, dk) = alg.generate_keypair(OsRng);
        FrodoKem976Keypair { secret: dk, public: ek }
    }
    pub fn public_key(&self) -> Vec<u8> {
        self.public.value().to_vec()
    }

    pub fn decapsulate(&self, ct: &[u8]) -> Vec<u8> {
        let alg = Algorithm::FrodoKem976Shake;
        let decap = alg
            .decapsulate(&self.secret, &alg.ciphertext_from_bytes(ct).unwrap())
            .unwrap();

        decap.0.value().to_vec()
    }

    pub fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let ek = Algorithm::FrodoKem976Shake.encryption_key_from_bytes(pk).unwrap();
        let (ct, ss) = Algorithm::FrodoKem976Shake.encapsulate_with_rng(&ek, &mut OsRng).unwrap();
        
        (ct.value().to_vec(), ss.value().to_vec())
    }
    pub fn pk_size(&self) -> usize {
        Algorithm::FrodoKem976Shake.params().encryption_key_length
    }

    pub fn ct_size(&self) -> usize {
        Algorithm::FrodoKem976Shake.params().ciphertext_length
    }
}

impl FrodoKem640Keypair {
    pub fn generate() -> Self {
        let alg = Algorithm::FrodoKem640Shake;
        let (ek, dk) = alg.generate_keypair(OsRng);
        FrodoKem640Keypair { secret: dk, public: ek }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.public.value().to_vec()
    }

    pub fn decapsulate(&self, ct: &[u8]) -> Vec<u8> {
        let alg = Algorithm::FrodoKem640Shake;
        let decap = alg
            .decapsulate(&self.secret, &alg.ciphertext_from_bytes(ct).unwrap())
            .unwrap();

        decap.0.value().to_vec()
    }

    pub fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let ek = Algorithm::FrodoKem640Shake.encryption_key_from_bytes(pk).unwrap();
        let (ct, ss) = Algorithm::FrodoKem640Shake.encapsulate_with_rng(&ek, &mut OsRng).unwrap();
        
        (ct.value().to_vec(), ss.value().to_vec())
    }

    pub fn pk_size(&self) -> usize {
        Algorithm::FrodoKem640Shake.params().encryption_key_length
    }

    pub fn ct_size(&self) -> usize {
        Algorithm::FrodoKem640Shake.params().ciphertext_length
    }
}

impl Ed25519Keypair {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let keypair = Ed25519SigningKey::from_bytes(&seed);

        Ed25519Keypair(keypair)
    }

    pub fn load_or_generate() -> Self {
        if let Ok(mut file) = File::open("serv_data/keys/Ed25519") {
            let mut seed = [0u8; 32];

            if file.read_exact(&mut seed).is_ok() {
                let keypair = Ed25519SigningKey::from_bytes(&seed);
                return Ed25519Keypair(keypair);
            }
        }

        println!("Warning: Generating new Ed25519 keypair (existing seed missing or invalid)");
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        let keypair = Ed25519SigningKey::from_bytes(&seed);

        if let Some(parent) = std::path::Path::new("serv_data/keys/Ed25519").parent() {
            create_dir_all(parent).expect("Failed to create key directory");
        }

        let mut file = File::create("serv_data/keys/Ed25519").expect("Failed to create key file");
        file.write_all(&seed).expect("Failed to write private seed");
        Ed25519Keypair(keypair)
    }
}


pub enum PqSigningKeys {
    // Falcon variants
    Falcon512(Falcon512Keypair),
    Falcon1024(Falcon1024Keypair),
    FalconPadded512(FalconPadded512Keypair),
    FalconPadded1024(FalconPadded1024Keypair),

    // ML-DSA variants
    MLDsa44(MLDsa44Keypair),
    MLDsa65(MLDsa65Keypair),
    MLDsa87(MLDsa87Keypair),

    // SPHINCS+ SHA2 variants
    SphincsSha2128f(SphincsSha2128fKeypair),
    SphincsSha2192f(SphincsSha2192fKeypair),
    SphincsSha2256f(SphincsSha2256fKeypair),
    SphincsSha2128s(SphincsSha2128sKeypair),
    SphincsSha2192s(SphincsSha2192sKeypair),
    SphincsSha2256s(SphincsSha2256sKeypair),

    // SPHINCS+ SHAKE variants
    SphincsShake128f(SphincsShake128fKeypair),
    SphincsShake192f(SphincsShake192fKeypair),
    SphincsShake256f(SphincsShake256fKeypair),
    SphincsShake128s(SphincsShake128sKeypair),
    SphincsShake192s(SphincsShake192sKeypair),
    SphincsShake256s(SphincsShake256sKeypair),
}


pub enum CSigningKeys {
    Ed25519(Ed25519Keypair)
}

impl CSigningKeys {
    pub fn public(&self) -> Vec<u8> {
        match self {
            CSigningKeys::Ed25519(ed25519) => ed25519.0.verifying_key().as_bytes().to_vec(),
        }
    }

    pub fn sign(&mut self, data: &Vec<u8>) -> Vec<u8> {
        match self {
            CSigningKeys::Ed25519(ed25519) => ed25519.0.sign(&data).to_vec()
        }
    }

    pub fn verify(&mut self, data: &[u8], pk: &[u8], signature: &[u8]) -> Result<(), Box<dyn Error>>  {
        match self {
            CSigningKeys::Ed25519(_) => {
                let pubkey_array: &[u8; 32] = pk.try_into().expect("slice with incorrect length");
                let verifying_key: VerifyingKey = VerifyingKey::from_bytes(pubkey_array)?;
                let signature: Ed25519Signature = Ed25519Signature::try_from(&signature[..])?;
                verifying_key.verify_strict(data, &signature)?;  
                
            }
        }
        Ok(())
    }

    pub fn pk_size(&self) -> usize {
        match self {
            CSigningKeys::Ed25519(_) => ED25519_PUBLIC_KEY_SIZE
        }
    } 

    pub fn sign_size(&self) -> usize {
        match self {
            CSigningKeys::Ed25519(_) => ED25519_SIGNATURE_SIZE
        }
    } 

    pub fn default() -> Self {
        let key = Ed25519Keypair::generate();
        CSigningKeys::Ed25519(key)
    }

    pub fn key_type(&self) -> String {
        match self {
            CSigningKeys::Ed25519(_) => "Ed25519".to_string()
        }

    }
}

impl PqSigningKeys {
    pub fn public(&self) -> Vec<u8> {
        match self {
            PqSigningKeys::Falcon512(k) => k.public_key(),
            PqSigningKeys::Falcon1024(k) => k.public_key(),
            PqSigningKeys::FalconPadded512(k) => k.public_key(),
            PqSigningKeys::FalconPadded1024(k) => k.public_key(),

            PqSigningKeys::MLDsa44(k) => k.public_key(),
            PqSigningKeys::MLDsa65(k) => k.public_key(),
            PqSigningKeys::MLDsa87(k) => k.public_key(),

            PqSigningKeys::SphincsSha2128f(k) => k.public_key(),
            PqSigningKeys::SphincsSha2192f(k) => k.public_key(),
            PqSigningKeys::SphincsSha2256f(k) => k.public_key(),
            PqSigningKeys::SphincsSha2128s(k) => k.public_key(),
            PqSigningKeys::SphincsSha2192s(k) => k.public_key(),
            PqSigningKeys::SphincsSha2256s(k) => k.public_key(),

            PqSigningKeys::SphincsShake128f(k) => k.public_key(),
            PqSigningKeys::SphincsShake192f(k) => k.public_key(),
            PqSigningKeys::SphincsShake256f(k) => k.public_key(),
            PqSigningKeys::SphincsShake128s(k) => k.public_key(),
            PqSigningKeys::SphincsShake192s(k) => k.public_key(),
            PqSigningKeys::SphincsShake256s(k) => k.public_key(),
        }
    }

    pub fn sign(&mut self, data: &Vec<u8>) -> Vec<u8> {
        match self {
            PqSigningKeys::Falcon512(k) => k.sign(data),
            PqSigningKeys::Falcon1024(k) => k.sign(data),
            PqSigningKeys::FalconPadded512(k) => k.sign(data),
            PqSigningKeys::FalconPadded1024(k) => k.sign(data),

            PqSigningKeys::MLDsa44(k) => k.sign(data),
            PqSigningKeys::MLDsa65(k) => k.sign(data),
            PqSigningKeys::MLDsa87(k) => k.sign(data),

            PqSigningKeys::SphincsSha2128f(k) => k.sign(data),
            PqSigningKeys::SphincsSha2192f(k) => k.sign(data),
            PqSigningKeys::SphincsSha2256f(k) => k.sign(data),
            PqSigningKeys::SphincsSha2128s(k) => k.sign(data),
            PqSigningKeys::SphincsSha2192s(k) => k.sign(data),
            PqSigningKeys::SphincsSha2256s(k) => k.sign(data),

            PqSigningKeys::SphincsShake128f(k) => k.sign(data),
            PqSigningKeys::SphincsShake192f(k) => k.sign(data),
            PqSigningKeys::SphincsShake256f(k) => k.sign(data),
            PqSigningKeys::SphincsShake128s(k) => k.sign(data),
            PqSigningKeys::SphincsShake192s(k) => k.sign(data),
            PqSigningKeys::SphincsShake256s(k) => k.sign(data),
        }
    }

    pub fn verify(&self, message: &[u8], public_key: &[u8], signature: &[u8]) -> Result<(), Box<dyn Error>> {
        match self {
            PqSigningKeys::Falcon512(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::Falcon1024(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::FalconPadded512(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::FalconPadded1024(k) => k.verify(message, public_key, signature)?,

            PqSigningKeys::MLDsa44(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::MLDsa65(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::MLDsa87(k) => k.verify(message, public_key, signature)?,

            PqSigningKeys::SphincsSha2128f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsSha2192f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsSha2256f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsSha2128s(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsSha2192s(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsSha2256s(k) => k.verify(message, public_key, signature)?,

            PqSigningKeys::SphincsShake128f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsShake192f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsShake256f(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsShake128s(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsShake192s(k) => k.verify(message, public_key, signature)?,
            PqSigningKeys::SphincsShake256s(k) => k.verify(message, public_key, signature)?,
        }

        Ok(())
    }

    pub fn pk_size(&self) -> usize {
        match self {
            PqSigningKeys::Falcon512(k) => k.pk_size(),
            PqSigningKeys::Falcon1024(k) => k.pk_size(),
            PqSigningKeys::FalconPadded512(k) => k.pk_size(),
            PqSigningKeys::FalconPadded1024(k) => k.pk_size(),

            PqSigningKeys::MLDsa44(k) => k.pk_size(),
            PqSigningKeys::MLDsa65(k) => k.pk_size(),
            PqSigningKeys::MLDsa87(k) => k.pk_size(),

            PqSigningKeys::SphincsSha2128f(k) => k.pk_size(),
            PqSigningKeys::SphincsSha2192f(k) => k.pk_size(),
            PqSigningKeys::SphincsSha2256f(k) => k.pk_size(),
            PqSigningKeys::SphincsSha2128s(k) => k.pk_size(),
            PqSigningKeys::SphincsSha2192s(k) => k.pk_size(),
            PqSigningKeys::SphincsSha2256s(k) => k.pk_size(),

            PqSigningKeys::SphincsShake128f(k) => k.pk_size(),
            PqSigningKeys::SphincsShake192f(k) => k.pk_size(),
            PqSigningKeys::SphincsShake256f(k) => k.pk_size(),
            PqSigningKeys::SphincsShake128s(k) => k.pk_size(),
            PqSigningKeys::SphincsShake192s(k) => k.pk_size(),
            PqSigningKeys::SphincsShake256s(k) => k.pk_size(),
        }
    }

    pub fn sign_size(&self) -> usize {
        match self {
            PqSigningKeys::Falcon512(k) => k.sign_size(),
            PqSigningKeys::Falcon1024(k) => k.sign_size(),
            PqSigningKeys::FalconPadded512(k) => k.sign_size(),
            PqSigningKeys::FalconPadded1024(k) => k.sign_size(),

            PqSigningKeys::MLDsa44(k) => k.sign_size(),
            PqSigningKeys::MLDsa65(k) => k.sign_size(),
            PqSigningKeys::MLDsa87(k) => k.sign_size(),

            PqSigningKeys::SphincsSha2128f(k) => k.sign_size(),
            PqSigningKeys::SphincsSha2192f(k) => k.sign_size(),
            PqSigningKeys::SphincsSha2256f(k) => k.sign_size(),
            PqSigningKeys::SphincsSha2128s(k) => k.sign_size(),
            PqSigningKeys::SphincsSha2192s(k) => k.sign_size(),
            PqSigningKeys::SphincsSha2256s(k) => k.sign_size(),

            PqSigningKeys::SphincsShake128f(k) => k.sign_size(),
            PqSigningKeys::SphincsShake192f(k) => k.sign_size(),
            PqSigningKeys::SphincsShake256f(k) => k.sign_size(),
            PqSigningKeys::SphincsShake128s(k) => k.sign_size(),
            PqSigningKeys::SphincsShake192s(k) => k.sign_size(),
            PqSigningKeys::SphincsShake256s(k) => k.sign_size(),
        }
    }

    pub fn key_type(&self) -> String {
        match self {
            PqSigningKeys::Falcon512(_) => "Falcon512",
            PqSigningKeys::Falcon1024(_) => "Falcon1024",
            PqSigningKeys::FalconPadded512(_) => "FalconPadded512",
            PqSigningKeys::FalconPadded1024(_) => "FalconPadded1024",

            PqSigningKeys::MLDsa44(_) => "MLDsa44",
            PqSigningKeys::MLDsa65(_) => "MLDsa65",
            PqSigningKeys::MLDsa87(_) => "MLDsa87",

            PqSigningKeys::SphincsSha2128f(_) => "SphincsSha2128f",
            PqSigningKeys::SphincsSha2192f(_) => "SphincsSha2192f",
            PqSigningKeys::SphincsSha2256f(_) => "SphincsSha2256f",
            PqSigningKeys::SphincsSha2128s(_) => "SphincsSha2128s",
            PqSigningKeys::SphincsSha2192s(_) => "SphincsSha2192s",
            PqSigningKeys::SphincsSha2256s(_) => "SphincsSha2256s",

            PqSigningKeys::SphincsShake128f(_) => "SphincsShake128f",
            PqSigningKeys::SphincsShake192f(_) => "SphincsShake192f",
            PqSigningKeys::SphincsShake256f(_) => "SphincsShake256f",
            PqSigningKeys::SphincsShake128s(_) => "SphincsShake128s",
            PqSigningKeys::SphincsShake192s(_) => "SphincsShake192s",
            PqSigningKeys::SphincsShake256s(_) => "SphincsShake256s",
        }
        .to_string()
    }

    pub fn default() -> Self {
        PqSigningKeys::MLDsa44(MLDsa44Keypair::generate())
    }
}

pub enum PqAKAKeys {
    MlKem1024(MlKem1024Keypair),
    MlKem768(MlKem768Keypair),
    MlKem512(MlKem512Keypair),
    FrodoKem1344(FrodoKem1344Keypair),
    FrodoKem976(FrodoKem976Keypair),
    FrodoKem640(FrodoKem640Keypair),
    Hqc256(Hqc256Keypair),
    Hqc192(Hqc192Keypair),
    Hqc128(Hqc128Keypair),
    McEliece348864(Box<McEliece348864Keypair>),
    McEliece348864f(Box<McEliece348864fKeypair>),
    McEliece460896(Box<McEliece460896Keypair>),
    McEliece460896f(Box<McEliece460896fKeypair>),
    McEliece6688128(Box<McEliece6688128Keypair>),
    McEliece6688128f(Box<McEliece6688128fKeypair>),
    McEliece8192128(Box<McEliece8192128Keypair>),
    McEliece8192128f(Box<McEliece8192128fKeypair>),
    McEliece6960119(Box<McEliece6960119Keypair>),
    McEliece6960119f(Box<McEliece6960119fKeypair>),
}


pub enum CAKAKeys {
    X25519(X25519Keypair),
}

impl CAKAKeys {
    pub fn public(&self) -> Vec<u8> {
        match self {
            CAKAKeys::X25519(x25519) => x25519.public.as_bytes().to_vec(),
        }
    }

    pub fn decapsulate(&mut self, pk: &[u8]) -> Vec<u8> {
        match self {
            CAKAKeys::X25519(x25519) => {
                let arr: [u8; 32] = pk.try_into().expect("Failed to convert slice to array");
                let public_key = X25519PublicKey::from(arr);

                // Move secret out of x25519 to use diffie_hellman
                let secret = std::mem::replace(&mut x25519.secret, EphemeralSecret::random_from_rng(rand::rngs::OsRng));

                secret.diffie_hellman(&public_key).as_ref().to_vec()
            }
        }
    }

    pub fn pk_size(&self) -> usize {
        match self {
            CAKAKeys::X25519(_) => X25519_PUBLIC_KEY_SIZE
        }
    } 

    pub fn default() -> Self {
        let key = X25519Keypair::generate();
        CAKAKeys::X25519(key)
    }
}

impl PqAKAKeys {
    pub fn public(&self) -> Vec<u8> {
        match self {
            PqAKAKeys::MlKem1024(mlkem) => mlkem.public_key(),
            PqAKAKeys::MlKem768(mlkem) => mlkem.public_key(),
            PqAKAKeys::MlKem512(mlkem) => mlkem.public_key(),

            PqAKAKeys::FrodoKem1344(frodo_kem) => frodo_kem.public_key(),
            PqAKAKeys::FrodoKem976(frodo_kem) => frodo_kem.public_key(),
            PqAKAKeys::FrodoKem640(frodo_kem) => frodo_kem.public_key(),

            PqAKAKeys::Hqc256(hqc) => hqc.public_key(),
            PqAKAKeys::Hqc192(hqc) => hqc.public_key(),
            PqAKAKeys::Hqc128(hqc) => hqc.public_key(),

            PqAKAKeys::McEliece348864(k) => k.public_key(),
            PqAKAKeys::McEliece348864f(k) => k.public_key(),
            PqAKAKeys::McEliece460896(k) => k.public_key(),
            PqAKAKeys::McEliece460896f(k) => k.public_key(),
            PqAKAKeys::McEliece6688128(k) => k.public_key(),
            PqAKAKeys::McEliece6688128f(k) => k.public_key(),
            PqAKAKeys::McEliece8192128(k) => k.public_key(),
            PqAKAKeys::McEliece8192128f(k) => k.public_key(),
            PqAKAKeys::McEliece6960119(k) => k.public_key(),
            PqAKAKeys::McEliece6960119f(k) => k.public_key(),
        }

    }

    pub fn decapsulate(&self, ct: &[u8]) -> Vec<u8> {
        match self {
            PqAKAKeys::MlKem1024(ml_kem) => ml_kem.decapsulate(ct),
            PqAKAKeys::MlKem768(ml_kem) => ml_kem.decapsulate(ct),
            PqAKAKeys::MlKem512(ml_kem) => ml_kem.decapsulate(ct),

            PqAKAKeys::Hqc256(hqc) => hqc.decapsulate(ct),
            PqAKAKeys::Hqc192(hqc) => hqc.decapsulate(ct),
            PqAKAKeys::Hqc128(hqc) => hqc.decapsulate(ct),

            PqAKAKeys::McEliece348864(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece348864f(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece460896(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece460896f(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece6688128(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece6688128f(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece8192128(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece8192128f(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece6960119(k) => k.decapsulate(ct),
            PqAKAKeys::McEliece6960119f(k) => k.decapsulate(ct),

            PqAKAKeys::FrodoKem1344(frodo_kem) => frodo_kem.decapsulate(ct),
            PqAKAKeys::FrodoKem976(frodo_kem) => frodo_kem.decapsulate(ct),
            PqAKAKeys::FrodoKem640(frodo_kem) => frodo_kem.decapsulate(ct),
        }
    }

    pub fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        match self {
            PqAKAKeys::MlKem1024(ml_kem) => ml_kem.encapsulate(pk),
            PqAKAKeys::MlKem768(ml_kem) => ml_kem.encapsulate(pk),
            PqAKAKeys::MlKem512(ml_kem) => ml_kem.encapsulate(pk),

            PqAKAKeys::Hqc256(hqc) => hqc.encapsulate(pk),
            PqAKAKeys::Hqc192(hqc) => hqc.encapsulate(pk),
            PqAKAKeys::Hqc128(hqc) => hqc.encapsulate(pk),

            PqAKAKeys::McEliece348864(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece348864f(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece460896(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece460896f(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece6688128(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece6688128f(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece8192128(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece8192128f(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece6960119(k) => k.encapsulate(pk),
            PqAKAKeys::McEliece6960119f(k) => k.encapsulate(pk),

            PqAKAKeys::FrodoKem1344(frodo_kem)  => frodo_kem.encapsulate(pk),
            PqAKAKeys::FrodoKem976(frodo_kem)  => frodo_kem.encapsulate(pk),
            PqAKAKeys::FrodoKem640(frodo_kem)  => frodo_kem.encapsulate(pk),
        }
    }
    pub fn default() -> Self {
        let key = MlKem1024Keypair::generate();
        PqAKAKeys::MlKem1024(key)
    }

    pub fn pk_size(&self) -> usize {
        match self {
            PqAKAKeys::MlKem1024(k) => k.pk_size(),
            PqAKAKeys::MlKem768(k) => k.pk_size(),
            PqAKAKeys::MlKem512(k) => k.pk_size(),

            PqAKAKeys::Hqc256(k) => k.pk_size(),
            PqAKAKeys::Hqc192(k) => k.pk_size(),
            PqAKAKeys::Hqc128(k) => k.pk_size(),

            PqAKAKeys::McEliece348864(k) => k.pk_size(),
            PqAKAKeys::McEliece348864f(k) => k.pk_size(),
            PqAKAKeys::McEliece460896(k) => k.pk_size(),
            PqAKAKeys::McEliece460896f(k) => k.pk_size(),
            PqAKAKeys::McEliece6688128(k) => k.pk_size(),
            PqAKAKeys::McEliece6688128f(k) => k.pk_size(),
            PqAKAKeys::McEliece8192128(k) => k.pk_size(),
            PqAKAKeys::McEliece8192128f(k) => k.pk_size(),
            PqAKAKeys::McEliece6960119(k) => k.pk_size(),
            PqAKAKeys::McEliece6960119f(k) => k.pk_size(),

            PqAKAKeys::FrodoKem1344(frodo_kem) => frodo_kem.pk_size(),
            PqAKAKeys::FrodoKem976(frodo_kem) => frodo_kem.pk_size(),
            PqAKAKeys::FrodoKem640(frodo_kem) => frodo_kem.pk_size(),
        }
    } 
    pub fn ct_size(&self) -> usize {
        match self {
            PqAKAKeys::MlKem1024(k) => k.ct_size(),
            PqAKAKeys::MlKem768(k) => k.ct_size(),
            PqAKAKeys::MlKem512(k) => k.ct_size(),

            PqAKAKeys::Hqc256(k) => k.ct_size(),
            PqAKAKeys::Hqc192(k) => k.ct_size(),
            PqAKAKeys::Hqc128(k) => k.ct_size(),

            PqAKAKeys::McEliece348864(k) => k.ct_size(),
            PqAKAKeys::McEliece348864f(k) => k.ct_size(),
            PqAKAKeys::McEliece460896(k) => k.ct_size(),
            PqAKAKeys::McEliece460896f(k) => k.ct_size(),
            PqAKAKeys::McEliece6688128(k) => k.ct_size(),
            PqAKAKeys::McEliece6688128f(k) => k.ct_size(),
            PqAKAKeys::McEliece8192128(k) => k.ct_size(),
            PqAKAKeys::McEliece8192128f(k) => k.ct_size(),
            PqAKAKeys::McEliece6960119(k) => k.ct_size(),
            PqAKAKeys::McEliece6960119f(k) => k.ct_size(),

            PqAKAKeys::FrodoKem1344(frodo_kem) => frodo_kem.ct_size(),
            PqAKAKeys::FrodoKem976(frodo_kem) =>  frodo_kem.ct_size(),
            PqAKAKeys::FrodoKem640(frodo_kem) =>  frodo_kem.ct_size(),
        }
    } 
}

pub struct PqTlsSettings {
    pub pq_signing_keys: PqSigningKeys,
    pub c_signing_keys: CSigningKeys,
    pub pq_aka_keys: PqAKAKeys,
    pub c_aka_keys: CAKAKeys
}


impl Default for PqTlsSettings {
    fn default() -> Self {
        Self {
            pq_signing_keys: PqSigningKeys::default(),
            c_signing_keys: CSigningKeys::default(),
            pq_aka_keys: PqAKAKeys::default(),
            c_aka_keys: CAKAKeys::default()
        }
    }
}