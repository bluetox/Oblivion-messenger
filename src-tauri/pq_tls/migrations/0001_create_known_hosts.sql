CREATE TABLE known_hosts (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  host             TEXT    NOT NULL,
  port             INTEGER NOT NULL,
  first_seen       TIMESTAMP NOT NULL,
  last_seen        TIMESTAMP NOT NULL,
  pc_sign_pk       BLOB    NOT NULL,
  pc_sign_pk_type  TEXT    NOT NULL,
  c_sign_pk        BLOB,
  c_sign_pk_type   TEXT,
  fingerprint      BLOB    NOT NULL,
  UNIQUE(host, port)
);
