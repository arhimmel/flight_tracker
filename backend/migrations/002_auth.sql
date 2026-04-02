CREATE TABLE IF NOT EXISTS users (
    id         TEXT PRIMARY KEY,         -- UUID v4
    email      TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS otp_tokens (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    email      TEXT NOT NULL,
    token_hash TEXT NOT NULL,            -- SHA-256 hex of the 6-digit code
    expires_at TEXT NOT NULL,            -- RFC 3339
    used       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sessions (
    id         TEXT PRIMARY KEY,         -- UUID v4
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,            -- RFC 3339, 30-day TTL
    created_at TEXT NOT NULL
);

-- Nullable so existing rows (if any) are unaffected.
ALTER TABLE alerts ADD COLUMN user_id TEXT;
