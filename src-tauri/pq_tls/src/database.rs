use std::path::Path;
use sqlx::{sqlite::SqlitePoolOptions, migrate::MigrateDatabase ,Sqlite, Pool};
use sqlx::migrate::Migrator;
use sqlx::FromRow;
use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, FromRow)]
pub struct KnownHost {
    pub id: i64,
    pub host: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub pq_sign_pk: Vec<u8>,
    pub pq_sign_pk_type: String,
    pub c_sign_pk: Vec<u8>,
    pub c_sign_pk_type: String,
    pub fingerprint: [u8; 32],
}

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn setup_db(path: &Path) -> Pool<Sqlite> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }
    
    let db_url = format!("sqlite://{}", path.display());

    Sqlite::create_database(&db_url).await.expect("Failed to create database");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    MIGRATOR.run(&pool).await.expect("Migration failed");

    pool
}


pub async fn insert_new_peer(db: &Pool<Sqlite>, pq_sign_pk: &[u8], pq_sign_pk_type: &str, c_sign_pk: &[u8], c_sign_pk_type: &str, fingerprint: &[u8; 32], host: &str, port: u16) -> Result<(), sqlx::Error> {
    let now = Utc::now();

    sqlx::query(
    r#"
    INSERT INTO known_hosts (host, port, first_seen, last_seen, pc_sign_pk, pc_sign_pk_type, c_sign_pk, c_sign_pk_type, fingerprint)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#
    )
    .bind(host)
    .bind(port)
    .bind(now)
    .bind(now)
    .bind(pq_sign_pk)
    .bind(pq_sign_pk_type)
    .bind(c_sign_pk)
    .bind(c_sign_pk_type)
    .bind(fingerprint.to_vec())
    .execute(db)
    .await?;

    Ok(())
}

pub async fn fingerprint_exists(db: &Pool<Sqlite>, fingerprint: &[u8; 32]) -> Result<bool, sqlx::Error> {
    let result: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM known_hosts WHERE fingerprint = ?
        "#
    )
    .bind(fingerprint.as_slice())
    .fetch_one(db)
    .await?;

    Ok(result.0 > 0)
}