CREATE TABLE chats (
    chat_id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
    chat_profil TEXT NOT NULL,
    chat_name TEXT NOT NULL,
    dst_user_id TEXT NOT NULL,
    shared_secret BLOB,
    last_updated BIGINT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(chat_profil, dst_user_id)
);

CREATE TABLE messages (
    message_id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
    chat_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    message_type TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id)
);

CREATE TABLE profiles (
    profile_id TEXT PRIMARY KEY NOT NULL DEFAULT (lower(hex(randomblob(16)))),
    profile_name TEXT,
    dilithium_public BLOB,
    dilithium_private BLOB,
    kyber_public BLOB,
    kyber_private BLOB,
    ed25519 BLOB,
    nonce BLOB,
    user_id TEXT,
    password_hash TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
