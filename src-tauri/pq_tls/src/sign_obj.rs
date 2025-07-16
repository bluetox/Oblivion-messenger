use pqcrypto::traits::sign::{PublicKey, SecretKey, DetachedSignature};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use pqcrypto::sign::*;

macro_rules! define_sign {
    (struct $Struct:ident, mod $modname:ident) => {
        pub struct $Struct {
            pub secret: $modname::SecretKey,
            pub public: $modname::PublicKey,
        }

        impl $Struct {
            pub fn generate() -> Self {
                let (public, secret) = $modname::keypair();
                Self { public, secret }
            }

            pub fn load_or_generate() -> Self {
                let path = format!("serv_data/keys/{}", stringify!($modname));
                if let Ok(mut file) = File::open(&path) {
                    let mut pk = vec![0u8; $modname::public_key_bytes()];
                    let mut sk = vec![0u8; $modname::secret_key_bytes()];

                    if file.read_exact(&mut pk).is_ok() && file.read_exact(&mut sk).is_ok() {
                        let public = $modname::PublicKey::from_bytes(&pk)
                            .expect(concat!(stringify!($modname), "::PublicKey::from_bytes failed"));
                        let secret = $modname::SecretKey::from_bytes(&sk)
                            .expect(concat!(stringify!($modname), "::SecretKey::from_bytes failed"));
                        return Self { public, secret };
                    }
                }

                let (public, secret) = $modname::keypair();

                if let Some(parent) = Path::new(&path).parent() {
                    create_dir_all(parent).expect("Failed to create key directory");
                }

                let mut file = File::create(&path).expect("Failed to create key file");
                file.write_all(public.as_bytes()).expect("Failed to write public key");
                file.write_all(secret.as_bytes()).expect("Failed to write secret key");

                Self { public, secret }
            }

            pub fn public_key(&self) -> Vec<u8> {
                self.public.as_bytes().to_vec()
            }

            pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
                let sig = $modname::detached_sign(msg, &self.secret);
                sig.as_bytes().to_vec()
            }

            pub fn verify(&self, msg: &[u8], public: &[u8], sig: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
                let signature = $modname::DetachedSignature::from_bytes(sig)
                    .map_err(|_| "Invalid signature format")?;
                let public = $modname::PublicKey::from_bytes(public)
                    .map_err(|_| "Invalid public key format")?;
                $modname::verify_detached_signature(&signature, msg, &public)
                    .map_err(|_| "Signature verification failed".into())
            }


            pub fn pk_size(&self) -> usize {
                $modname::public_key_bytes()
            }

            pub fn sign_size(&self) -> usize {
                $modname::signature_bytes()
            }
        }
    };
}

// Falcon variants
define_sign!(struct Falcon512Keypair, mod falcon512);
define_sign!(struct Falcon1024Keypair, mod falcon1024);
define_sign!(struct FalconPadded512Keypair, mod falconpadded512);
define_sign!(struct FalconPadded1024Keypair, mod falconpadded1024);

// ML-DSA variants
define_sign!(struct MLDsa44Keypair, mod mldsa44);
define_sign!(struct MLDsa65Keypair, mod mldsa65);
define_sign!(struct MLDsa87Keypair, mod mldsa87);

// SPHINCS+ SHA2 variants
define_sign!(struct SphincsSha2128fKeypair, mod sphincssha2128fsimple);
define_sign!(struct SphincsSha2192fKeypair, mod sphincssha2192fsimple);
define_sign!(struct SphincsSha2256fKeypair, mod sphincssha2256fsimple);
define_sign!(struct SphincsSha2128sKeypair, mod sphincssha2128ssimple);
define_sign!(struct SphincsSha2192sKeypair, mod sphincssha2192ssimple);
define_sign!(struct SphincsSha2256sKeypair, mod sphincssha2256ssimple);

// SPHINCS+ SHAKE variants
define_sign!(struct SphincsShake128fKeypair, mod sphincsshake128fsimple);
define_sign!(struct SphincsShake192fKeypair, mod sphincsshake192fsimple);
define_sign!(struct SphincsShake256fKeypair, mod sphincsshake256fsimple);
define_sign!(struct SphincsShake128sKeypair, mod sphincsshake128ssimple);
define_sign!(struct SphincsShake192sKeypair, mod sphincsshake192ssimple);
define_sign!(struct SphincsShake256sKeypair, mod sphincsshake256ssimple);
