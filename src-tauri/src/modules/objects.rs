pub struct Keys {
    pub dilithium_keys: pqc_dilithium::Keypair,
    pub ed25519_keys: ring::signature::Ed25519KeyPair,
    pub kyber_keys: pqc_kyber::Keypair,
    pub nonce: [u8; 16],
}

pub struct AppState {
    pub db: Db,
}
pub type Db = sqlx::Pool<sqlx::Sqlite>;

