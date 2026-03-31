# Flight Tracker — Claude Code Guide

## Project Overview

A personal flight price monitoring tool. Users register a flight with a target price; the system polls for the current price on a configurable interval and notifies in-app when the price drops.

For all architectural decisions and the rationale behind them, see **[ARCHITECTURE.md](./ARCHITECTURE.md)**.

---

## Structure

```
flight_tracker/
├── ARCHITECTURE.md       # Architectural decision records
├── CLAUDE.md             # This file
├── backend/              # Rust / Axum HTTP server + background poller
│   ├── Cargo.toml
│   ├── .env
│   ├── migrations/       # sqlx migrations (run automatically on startup)
│   └── src/
│       ├── main.rs       # Entrypoint — wires router, DB, poller, SSE channel
│       ├── db.rs         # SQLite pool init
│       ├── models.rs     # Shared structs (Alert, AlertEvent, etc.)
│       ├── routes/       # alerts.rs (REST CRUD), sse.rs (SSE stream)
│       └── poller/       # Polling loop + PriceFetcher trait + providers
│           ├── price_fetcher.rs   # Trait definition — do not couple to any provider here
│           └── providers/         # One file per provider (mock.rs, kiwi.rs, …)
└── frontend/             # Next.js 14 App Router
    ├── app/              # Pages and layout
    ├── components/       # AlertCard, AlertForm, AlertList
    ├── hooks/            # useAlerts, useSSE
    └── lib/              # api.ts (fetch helpers), types.ts
```

---

## Running

### Docker (recommended)

```bash
docker compose up --build
```

- Frontend: http://localhost:3000
- Backend:  http://localhost:8080

The SQLite database is persisted in a named Docker volume (`db_data`).

To use a real price provider, edit the `backend` service environment in `docker-compose.yml`:
```yaml
PRICE_PROVIDER: kiwi
KIWI_API_KEY: your_key_here
```

### Local Development

**Backend**
```bash
cd backend
cargo run
# Defaults: MockProvider, port 8080, SQLite at data/tracker.db
```

**Frontend**
```bash
cd frontend
npm install
npm run dev
# Runs at http://localhost:3000
```

---

## Environment Variables

### Backend (`backend/.env`)

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite://data/tracker.db` | SQLite file path |
| `PORT` | `8080` | HTTP server port |
| `POLL_INTERVAL_MINS` | `30` | How often to check prices |
| `PRICE_PROVIDER` | `mock` | Active provider: `mock` or `kiwi` |
| `KIWI_API_KEY` | — | Required when `PRICE_PROVIDER=kiwi` |

### Frontend (`frontend/.env.local`)

| Variable | Default | Description |
|---|---|---|
| `NEXT_PUBLIC_API_URL` | `http://localhost:8080` | Backend base URL |

---

## Adding a New Price Provider

1. Create `backend/src/poller/providers/<name>.rs`
2. Implement the `PriceFetcher` trait from `poller/price_fetcher.rs`:
   ```rust
   #[async_trait]
   impl PriceFetcher for MyProvider {
       async fn fetch_price(&self, flight_number, origin, destination, date) -> Result<PriceResult>
   }
   ```
3. Export it in `poller/providers/mod.rs`
4. Add a match arm in `main.rs` under the `PRICE_PROVIDER` env var block
5. Document it in the providers table in `ARCHITECTURE.md` (ADR-004)

The polling loop, SSE notifications, and database logic require no changes.

---

## Key Conventions

- **Poller is provider-agnostic.** `poller/mod.rs` depends only on `Arc<dyn PriceFetcher>`. Never import a concrete provider there.
- **SQLite is the only data store.** No Redis, no external cache. Price caching uses the `price_cache` table with a 30-minute TTL enforced in application logic.
- **SSE for push, REST for CRUD.** Do not introduce WebSockets. See ADR-006.
- **MockProvider is the default.** The system must be fully runnable without any API key for local development and testing.
- **Migrations run automatically.** `sqlx::migrate!` fires at startup. Add new migrations as numbered files in `backend/migrations/`.
