# Cryptifier

A Rust service that watches cryptocurrency prices and sends **Telegram alerts**
when a coin moves more than a configured percentage — reporting the **actual**
measured change, not the threshold.

## What it does

- Checks configured currencies (default: `bitcoin`, `ethereum`), then sleeps
  `APP__NOTIFIER_SLEEP` seconds (default 300) before the next check.
- Computes the real percentage move between checks via `get_price_change`.
- When the move exceeds a coin's `percentage_threshold`, it builds a message
  like `💰 <b>Bitcoin</b> is up 5.40%! $71000` and sends it to Telegram.
- For `bitcoin`, the message also includes 24h volume, reachable Bitcoin nodes,
  the Fear & Greed index, and the CBBI (ColinTalksCrypto) index.
- All currencies are checked **concurrently** (`tokio::task::JoinSet`), and every
  HTTP call shares **one `reqwest::Client`** connection pool (`src/http.rs`).

## Architecture

| Module | Purpose |
|---|---|
| `src/notifier.rs` | Orchestration loop: price checks + Telegram notifications |
| `src/get_price_change.rs` | Tracks last-seen prices, returns `(PriceChange, actual %)` |
| `src/config.rs` | Reads env config, defaults to bitcoin/ethereum at 2.0% |
| `src/cache.rs` | Generic `Cache<T>` with TTL |
| `src/cached_api.rs` | Wraps sources with short (10 min) / long (12 h) caches |
| `src/http.rs` | Single shared `reqwest::Client` for all API calls |
| `src/shutdown.rs` | `Shutdown` flag + `sleep_or_shutdown` — stops the loop between cycles |
| `src/sources/` | CoinGecko, bitdis.org, alternative.me (F&G), colintalkscrypto (CBBI) |
| `src/notifiers/telegram.rs` | Telegram bot: `notify(ticker, text)` |

## Configuration (env vars)

| Var | Default | Purpose |
|---|---|---|
| `APP__CURRENCIES` | `[{"ticker":"bitcoin","percentage_threshold":2.0},{"ticker":"ethereum","percentage_threshold":2.0}]` | JSON list of coins + move threshold (%) |
| `APP__NOTIFIER_SLEEP` | `300` | Seconds to sleep once a check has finished. Anything outside `1` to `86400` inclusive (a day), or unparseable, falls back to `300` |
| `APP__TELEGRAM_API_KEY` | — | Telegram bot token |
| `APP__TELEGRAM_CHAT_IDS` | — | Chat IDs to notify, comma-separated. At most 30 are attempted per alert; any surplus is reported as an error rather than sent |
| `APP__TELEGRAM_GET_UPDATES` | `false` | Also discover chat IDs from Telegram `getUpdates` (cached 10 min) |
| `APP__QUIET_MODE_ENABLED` | `false` | Suppress alerts during a window |
| `APP__QUIET_MODE_START_HOUR` / `END_HOUR` | `0` / `6` | Quiet window (24h hours); equal hours means all day |
| `APP__LOG_LEVEL` | `info` | A level (`off`, `error`, `warn`, `info`, `debug`, `trace`, any case, or the number `0`–`5`) or `target=level` pairs such as `tokio=trace`. A bare module name is rejected rather than installed, because it silences every log line |

Config is read once at startup from the process env — set these before running.
Startup then logs what is genuinely in effect: the `.env` path tried, the log
level, and the notifier sleep. A value that was set but could not be used is an
ERROR line naming it, followed by the one actually in effect.

## Data sources

- [CoinGecko](https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_vol=true) — price + 24h volume
- [bitdis.org](https://bitdis.org/api/live-data) — reachable Bitcoin nodes
- [alternative.me](https://api.alternative.me/fng/) — Fear & Greed index
- [colintalkscrypto](https://www.colintalkscrypto.com/cbbi/) — CBBI index

## Testing

Integration-style tests live in `tests/`:

- `tests/tests.rs` — declares `unit` and `helpers` modules
- `tests/unit/` — unit tests for price-change detection, quiet-mode, the cache,
  config defaults, the notification formatter, log-level resolution, and the
  shutdown flag
- `tests/helpers.rs` + `tests/data/` — fixture helpers for the network sources

Run them with:

```bash
cargo test
```

## Build & run

```bash
# build
cargo build

# run (set env vars first)
APP__TELEGRAM_API_KEY=... APP__TELEGRAM_CHAT_IDS=... cargo run
```

Toolchain is pinned by `rust-toolchain.toml` — rustup installs and switches to it
automatically. The `Dockerfile`'s `FROM rust:` tag is the container-side pin and
must be bumped to match.

Or via Docker (`Dockerfile` installs `procps` for the healthcheck and runs
tzdata non-interactively).

## Stopping the service

Send `SIGTERM` (`docker stop`) or `SIGINT` (Ctrl+C). The cycle in flight is
allowed to finish — the loop only stops **between** cycles, so no price check or
Telegram send is cancelled half-way — and the process then exits `0`.

Draining takes real time. A worst-case cycle burns ~25 s of HTTP timeouts:
CoinGecko 5 s, the bitcoin sources' `join!` 10 s, Telegram's `getUpdates` 5 s (only
on a cache older than 10 min), then 5 s for the messages — sent concurrently, and
bounded to 30 per alert, so the count of chat IDs cannot add to it. `docker-compose.yml` therefore sets
`stop_grace_period: 45s`; without it Docker kills the container at its 10 s
default and the exit code is `137`. Compose only — a container started with a
plain `docker run` needs `docker stop -t 45 <name>` instead.

Re-check that budget if any per-request timeout changes; it is derived, not
measured.

A second signal during the drain is not observed: the wait either finishes or it
is killed.
