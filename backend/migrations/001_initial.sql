CREATE TABLE IF NOT EXISTS alerts (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    flight_number TEXT    NOT NULL,
    flight_date   TEXT    NOT NULL,  -- ISO 8601: "2026-04-15"
    origin        TEXT    NOT NULL,  -- IATA code e.g. "JFK"
    destination   TEXT    NOT NULL,  -- IATA code e.g. "LAX"
    target_price  REAL    NOT NULL,
    current_price REAL,
    status        TEXT    NOT NULL DEFAULT 'active', -- active | triggered | expired
    created_at    TEXT    NOT NULL,
    last_checked  TEXT,
    notified_at   TEXT
);

CREATE TABLE IF NOT EXISTS price_cache (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    origin       TEXT NOT NULL,
    destination  TEXT NOT NULL,
    flight_date  TEXT NOT NULL,
    price        REAL NOT NULL,
    fetched_at   TEXT NOT NULL,
    UNIQUE(origin, destination, flight_date)
);

CREATE TABLE IF NOT EXISTS price_history (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    alert_id   INTEGER NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    price      REAL    NOT NULL,
    checked_at TEXT    NOT NULL
);
