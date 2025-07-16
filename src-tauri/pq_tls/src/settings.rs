use crate::objects::*;
use crate::kem_obj::*;
use crate::sign_obj::*;
pub fn bytes_to_settings(settings_bytes: &[u8]) -> PqTlsSettings {
    let pq_signing_keys = match settings_bytes.get(0) {
        Some(0) => PqSigningKeys::Falcon512(Falcon512Keypair::load_or_generate()),
        Some(1) => PqSigningKeys::Falcon1024(Falcon1024Keypair::load_or_generate()),
        Some(2) => PqSigningKeys::FalconPadded512(FalconPadded512Keypair::load_or_generate()),
        Some(3) => PqSigningKeys::FalconPadded1024(FalconPadded1024Keypair::load_or_generate()),

        Some(4) => PqSigningKeys::MLDsa44(MLDsa44Keypair::load_or_generate()),
        Some(5) => PqSigningKeys::MLDsa65(MLDsa65Keypair::load_or_generate()),
        Some(6) => PqSigningKeys::MLDsa87(MLDsa87Keypair::load_or_generate()),

        Some(7) => PqSigningKeys::SphincsSha2128f(SphincsSha2128fKeypair::load_or_generate()),
        Some(8) => PqSigningKeys::SphincsSha2192f(SphincsSha2192fKeypair::load_or_generate()),
        Some(9) => PqSigningKeys::SphincsSha2256f(SphincsSha2256fKeypair::load_or_generate()),
        Some(10) => PqSigningKeys::SphincsSha2128s(SphincsSha2128sKeypair::load_or_generate()),
        Some(11) => PqSigningKeys::SphincsSha2192s(SphincsSha2192sKeypair::load_or_generate()),
        Some(12) => PqSigningKeys::SphincsSha2256s(SphincsSha2256sKeypair::load_or_generate()),

        Some(13) => PqSigningKeys::SphincsShake128f(SphincsShake128fKeypair::load_or_generate()),
        Some(14) => PqSigningKeys::SphincsShake192f(SphincsShake192fKeypair::load_or_generate()),
        Some(15) => PqSigningKeys::SphincsShake256f(SphincsShake256fKeypair::load_or_generate()),
        Some(16) => PqSigningKeys::SphincsShake128s(SphincsShake128sKeypair::load_or_generate()),
        Some(17) => PqSigningKeys::SphincsShake192s(SphincsShake192sKeypair::load_or_generate()),
        Some(18) => PqSigningKeys::SphincsShake256s(SphincsShake256sKeypair::load_or_generate()),

        _ => panic!("pq_signing setting not handled"),
    };


    let c_signing_keys = match settings_bytes.get(1) {
        Some(0) => CSigningKeys::Ed25519(Ed25519Keypair::load_or_generate()),
        _ => CSigningKeys::Ed25519(Ed25519Keypair::load_or_generate()),
    };

    let pq_aka_keys = match settings_bytes.get(2) {
        Some(&0) => PqAKAKeys::MlKem512(MlKem512Keypair::generate()),
        Some(&1) => PqAKAKeys::MlKem768(MlKem768Keypair::generate()),
        Some(&2) => PqAKAKeys::MlKem1024(MlKem1024Keypair::generate()),
        Some(&3) => PqAKAKeys::FrodoKem1344(FrodoKem1344Keypair::generate()),
        Some(&4) => PqAKAKeys::FrodoKem976(FrodoKem976Keypair::generate()),
        Some(&5) => PqAKAKeys::FrodoKem640(FrodoKem640Keypair::generate()),
        Some(&6) => PqAKAKeys::Hqc256(Hqc256Keypair::generate()),
        Some(&7) => PqAKAKeys::Hqc192(Hqc192Keypair::generate()),
        Some(&8) => PqAKAKeys::Hqc128(Hqc128Keypair::generate()),
        Some(&9) => PqAKAKeys::McEliece6960119(Box::new(McEliece6960119Keypair::generate())),
        Some(&10) => PqAKAKeys::McEliece6960119f(Box::new(McEliece6960119fKeypair::generate())),
        Some(&11) => PqAKAKeys::McEliece348864(Box::new(McEliece348864Keypair::generate())),
        Some(&12) => PqAKAKeys::McEliece348864f(Box::new(McEliece348864fKeypair::generate())),
        Some(&13) => PqAKAKeys::McEliece460896(Box::new(McEliece460896Keypair::generate())),
        Some(&14) => PqAKAKeys::McEliece460896f(Box::new(McEliece460896fKeypair::generate())),
        Some(&15) => PqAKAKeys::McEliece6688128(Box::new(McEliece6688128Keypair::generate())),
        Some(&16) => PqAKAKeys::McEliece6688128f(Box::new(McEliece6688128fKeypair::generate())),
        Some(&17) => PqAKAKeys::McEliece8192128(Box::new(McEliece8192128Keypair::generate())),
        Some(&18) => PqAKAKeys::McEliece8192128f(Box::new(McEliece8192128fKeypair::generate())),

        _ => {
            panic!("pq_aka setting not handled");
        }
    };


    let c_aka_keys = match settings_bytes.get(3) {
        Some(0) => CAKAKeys::X25519(X25519Keypair::generate()),
        _ => CAKAKeys::X25519(X25519Keypair::generate()),
    };

    PqTlsSettings {
        pq_signing_keys,
        c_signing_keys,
        pq_aka_keys,
        c_aka_keys
    }
}

pub fn settings_to_bytes(settings: &PqTlsSettings) -> [u8; 8] {
    let mut settings_bytes = [0u8; 8];
    settings_bytes[0] = match settings.pq_signing_keys {
        PqSigningKeys::Falcon512(_)         => 0,
        PqSigningKeys::Falcon1024(_)        => 1,
        PqSigningKeys::FalconPadded512(_)   => 2,
        PqSigningKeys::FalconPadded1024(_)  => 3,
        
        PqSigningKeys::MLDsa44(_)           => 4,
        PqSigningKeys::MLDsa65(_)           => 5,
        PqSigningKeys::MLDsa87(_)           => 6,
        
        PqSigningKeys::SphincsSha2128f(_)   => 7,
        PqSigningKeys::SphincsSha2192f(_)   => 8,
        PqSigningKeys::SphincsSha2256f(_)   => 9,
        PqSigningKeys::SphincsSha2128s(_)   => 10,
        PqSigningKeys::SphincsSha2192s(_)   => 11,
        PqSigningKeys::SphincsSha2256s(_)   => 12,
        
        PqSigningKeys::SphincsShake128f(_)  => 13,
        PqSigningKeys::SphincsShake192f(_)  => 14,
        PqSigningKeys::SphincsShake256f(_)  => 15,
        PqSigningKeys::SphincsShake128s(_)  => 16,
        PqSigningKeys::SphincsShake192s(_)  => 17,
        PqSigningKeys::SphincsShake256s(_)  => 18,
    };
    
    settings_bytes[1] = match settings.c_signing_keys {
        CSigningKeys::Ed25519(_) => 0
    };
    settings_bytes[2] = match settings.pq_aka_keys {
        PqAKAKeys::MlKem512(_) => 0,
        PqAKAKeys::MlKem768(_) => 1,
        PqAKAKeys::MlKem1024(_) => 2,
        PqAKAKeys::FrodoKem1344(_) => 3,
        PqAKAKeys::FrodoKem976(_) => 4,
        PqAKAKeys::FrodoKem640(_) => 5,
        PqAKAKeys::Hqc256(_) => 6,
        PqAKAKeys::Hqc192(_) => 7,
        PqAKAKeys::Hqc128(_) => 8,
        PqAKAKeys::McEliece348864(_) => 9,
        PqAKAKeys::McEliece348864f(_) => 10,
        PqAKAKeys::McEliece460896(_) => 11,
        PqAKAKeys::McEliece460896f(_) => 12,
        PqAKAKeys::McEliece6688128(_) => 13,
        PqAKAKeys::McEliece6688128f(_) => 14,
        PqAKAKeys::McEliece8192128(_) => 15,
        PqAKAKeys::McEliece8192128f(_) => 16,
        PqAKAKeys::McEliece6960119(_) => 17,
        PqAKAKeys::McEliece6960119f(_) => 18,
    };

    settings_bytes[3] = match settings.c_aka_keys {
        CAKAKeys::X25519(_) => 0
    };


    settings_bytes
}