# Architectural Decision Record — Flight Tracker

> **Project:** Flight Price Tracker
> **Date:** 2026-03-31
> **Status:** Active

---

## Overview

A personal flight price monitoring tool. The user registers a flight (number, date, origin, destination) with a target price. The system polls for the current price on a configurable interval and notifies the user in-app when the price drops at or below their target.

---

## ADR-001 — Backend Language: Rust (Axum)

**Decision:** Use Rust with the Axum web framework for the backend.

**Alternatives considered:**
- Node.js / Express
- Python / FastAPI
- Go / Gin

**Rationale:**
- Axum integrates natively with Tokio, which is the natural fit for a long-running background polling loop alongside an HTTP server — both live in the same async runtime with no external process management.
- Rust's type system enforces correctness at compile time, reducing runtime surprises in background jobs that run unattended.
- Low memory footprint suits a personal/self-hosted tool running continuously.

---

## ADR-002 — Frontend Framework: Next.js 14 (App Router)

**Decision:** Use Next.js 14 with the App Router and TypeScript.

**Alternatives considered:**
- Plain React (Vite)
- SvelteKit
- Remix

**Rationale:**
- Next.js App Router provides a clean server/client component boundary with minimal boilerplate.
- TypeScript gives end-to-end type safety when consuming the Rust API's JSON responses.
- Broad ecosystem and familiarity reduce setup friction for a focused personal tool.

---

## ADR-003 — Database: SQLite via sqlx

**Decision:** Use SQLite as the sole data store, accessed through `sqlx` with compile-time checked queries.

**Alternatives considered:**
- PostgreSQL
- Redis (for caching only)
- In-memory store

**Rationale:**
- SQLite is zero-ops — a single file, no server process, trivially backed up. Appropriate for a personal tool with a small number of alerts.
- `sqlx` provides async support, compile-time query validation, and a migration runner (`sqlx::migrate!`) that runs automatically on startup.
- Migrating to PostgreSQL later requires only swapping the `sqlx` feature flag and `DATABASE_URL`; no application logic changes.

**Schema decisions:**
- `alerts` — core entity, tracks status lifecycle: `active → triggered | expired`.
- `price_cache` — deduplicates external API calls across alerts sharing the same route and date; TTL enforced in application logic (30 min default).
- `price_history` — append-only log of every price check per alert, enabling future price trend charts.

---

## ADR-004 — Price Provider Abstraction: `PriceFetcher` Trait

**Decision:** Define a `PriceFetcher` async trait that the polling loop depends on. Concrete provider implementations live under `poller/providers/` and are wired in at startup via the `PRICE_PROVIDER` environment variable.

**Alternatives considered:**
- Hardcoding a single provider
- Configuration-file-driven provider selection
- Multiple providers queried simultaneously (price aggregation)

**Rationale:**
- The flight price data market is fragmented and volatile — APIs change pricing, rate limits, and data contracts frequently. Locking the core polling logic to a specific provider would make future migrations costly.
- The trait boundary (`fetch_price(flight, origin, destination, date) -> Result<PriceResult>`) is narrow and stable. Adding a provider is a single new file; the poller never changes.
- A `MockProvider` ships as a first-class implementation, making the entire system runnable in development and tests without any external API key or network call.

**Current providers:**

| Provider | File | Status |
|---|---|---|
| Mock (random price) | `providers/mock.rs` | Default, dev/test |
| Kiwi Tequila API | `providers/kiwi.rs` | Ready, requires `KIWI_API_KEY` |

---

## ADR-005 — Polling Mechanism: In-Process Tokio Task

**Decision:** Run the price polling loop as a long-lived `tokio::spawn` task inside the same process as the Axum HTTP server.

**Alternatives considered:**
- External cron job (crontab / systemd timer)
- Message queue (Redis pub/sub, RabbitMQ)
- Separate polling microservice

**Rationale:**
- For a personal tool, a separate scheduler process or message broker is operationally disproportionate.
- Tokio's `interval` primitive handles the periodic wake-up with no external dependencies.
- The polling loop shares the SQLite pool and the SSE broadcast channel directly with the HTTP layer via `Arc` — no serialization or IPC required.
- The interval is configurable via `POLL_INTERVAL_MINS`; the default is 30 minutes to stay within free-tier API quotas.

**Concurrency:** Individual alert checks within a poll cycle are fanned out with `futures::future::join_all`, so a slow API response for one alert does not delay others.

---

## ADR-006 — Real-Time Notifications: Server-Sent Events (SSE)

**Decision:** Use SSE (`GET /events`) to push price drop notifications from the Rust backend to the Next.js frontend.

**Alternatives considered:**
- WebSockets
- Polling from the frontend (short-poll / long-poll)
- Push notifications (Web Push API)

**Rationale:**
- Communication is strictly server-to-client — there is no need for the frontend to send data over the persistent connection. SSE is the correct primitive for this pattern.
- SSE works over plain HTTP/1.1, requires no upgrade handshake, and auto-reconnects natively in the browser via `EventSource`.
- Axum provides `axum::response::sse::Sse` out of the box. The backend uses a `tokio::sync::broadcast` channel to fan out events from the poller to all connected SSE clients.
- WebSockets would add bidirectional complexity with no benefit here.

---

## ADR-007 — Frontend-Backend Communication: REST + SSE

**Decision:** Use REST for CRUD operations on alerts and SSE for real-time push events. No GraphQL, tRPC, or gRPC.

**Rationale:**
- The API surface is small (create / list / delete alerts, one SSE stream). REST over JSON is the least complex approach that satisfies all requirements.
- Adding a query layer (GraphQL/tRPC) would introduce schema management overhead and additional dependencies for minimal gain.

**API surface:**

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/alerts` | Create a new price alert |
| `GET` | `/alerts` | List all alerts |
| `DELETE` | `/alerts/:id` | Remove an alert |
| `GET` | `/events` | SSE stream — price drop notifications |

---

## ADR-008 — Styling: Tailwind CSS

**Decision:** Use Tailwind CSS for all frontend styling.

**Rationale:**
- Utility-first approach eliminates the need to maintain a separate stylesheet for a focused single-page tool.
- Co-located styles make component structure immediately readable.
- No design system or component library dependency — keeps the frontend lean.

---

## ADR-009 — Notification Deduplication

**Decision:** Store `notified_at` on the `alerts` row and use `COALESCE(notified_at, ?)` on update so that the timestamp is set only once, on first trigger.

**Rationale:**
- Without deduplication, every subsequent poll cycle where `current_price <= target_price` would re-fire the SSE event and potentially re-send an email.
- Once an alert is `triggered`, it remains in that status and is excluded from future polling cycles (`WHERE status = 'active'`), giving a natural idempotency boundary.

---

## Summary of Key Trade-offs

| Decision | Chosen | Main trade-off accepted |
|---|---|---|
| Backend | Rust / Axum | Higher initial setup vs. Node/Python; pays off in runtime reliability |
| Database | SQLite | Not suitable for multi-user / high-concurrency; fine for personal use |
| Polling | In-process Tokio task | Process restart loses the next scheduled tick; acceptable for personal tool |
| Price provider | Abstracted trait | Slight indirection overhead; future-proofs against API churn |
| Notifications | SSE | Requires open browser tab; no background push |
| API style | REST + SSE | Less type-safe than tRPC; simpler to implement and maintain |
