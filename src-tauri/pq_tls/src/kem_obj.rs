use pqcrypto::kem::*;
use pqcrypto::traits::kem::{Ciphertext, PublicKey, SharedSecret};


macro_rules! define_kem {
    (struct $Struct:ident, mod $modname:ident) => {
        pub struct $Struct {
            pub secret: Box<$modname::SecretKey>,
            pub public: Box<$modname::PublicKey>,
        }

        impl $Struct {
            pub fn generate() -> Self {
                let (public, secret) = $modname::keypair();
                Self {
                    secret: Box::new(secret),
                    public: Box::new(public),
                }
            }

            pub fn public_key(&self) -> Vec<u8> {
                self.public.as_bytes().to_vec()
            }

            pub fn decapsulate(&self, ct: &[u8]) -> Vec<u8> {
                let c = $modname::Ciphertext::from_bytes(ct)
                    .expect(concat!(stringify!($modname), "::Ciphertext::from_bytes failed"));
                let ss = $modname::decapsulate(&c, &*self.secret);
                ss.as_bytes().to_vec()
            }

            pub fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
                let public_key = $modname::PublicKey::from_bytes(pk)
                    .expect(concat!(stringify!($modname), "::PublicKey::from_bytes failed"));
                let (ss, ct) = $modname::encapsulate(&public_key);
                (ct.as_bytes().to_vec(), ss.as_bytes().to_vec())
            }

            pub fn pk_size(&self) -> usize {
                $modname::public_key_bytes()
            }

            pub fn ct_size(&self) -> usize {
                $modname::ciphertext_bytes()
            }
        }
    };
}

// invoke for all your non‐McEliece KEMs:
define_kem!(struct Hqc256Keypair,    mod hqc256);
define_kem!(struct Hqc192Keypair,    mod hqc192);
define_kem!(struct Hqc128Keypair,    mod hqc128);
define_kem!(struct MlKem1024Keypair, mod mlkem1024);
define_kem!(struct MlKem768Keypair,  mod mlkem768);
define_kem!(struct MlKem512Keypair,  mod mlkem512);

define_kem!(struct McEliece348864Keypair,   mod mceliece348864);
define_kem!(struct McEliece348864fKeypair,  mod mceliece348864f);
define_kem!(struct McEliece460896Keypair,   mod mceliece460896);
define_kem!(struct McEliece460896fKeypair,  mod mceliece460896f);
define_kem!(struct McEliece6688128Keypair,  mod mceliece6688128);
define_kem!(struct McEliece6688128fKeypair, mod mceliece6688128f);
define_kem!(struct McEliece8192128Keypair,  mod mceliece8192128);
define_kem!(struct McEliece8192128fKeypair, mod mceliece8192128f);
define_kem!(struct McEliece6960119Keypair,  mod mceliece6960119);
define_kem!(struct McEliece6960119fKeypair, mod mceliece6960119f);
