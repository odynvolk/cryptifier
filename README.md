# Cryptifier

A Rust service that watches cryptocurrency prices and sends **Telegram alerts**
when a coin moves more than a configured percentage — reporting the **actual**
measured change, not the threshold.

## What it does

- Polls configured currencies (default: `bitcoin`, `ethereum`) every
  `APP__NOTIFIER_SLEEP` seconds (default 300).
- Computes the real percentage move between polls via `get_price_change`.
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
| `src/sources/` | CoinGecko, bitdis.org, alternative.me (F&G), colintalkscrypto (CBBI) |
| `src/notifiers/telegram.rs` | Telegram bot: `notify(ticker, text)` |

## Configuration (env vars)

| Var | Default | Purpose |
|---|---|---|
| `APP__CURRENCIES` | `[{"ticker":"bitcoin","percentage_threshold":2.0},{"ticker":"ethereum","percentage_threshold":2.0}]` | JSON list of coins + move threshold (%) |
| `APP__NOTIFIER_SLEEP` | `300` | Seconds between checks |
| `APP__TELEGRAM_API_KEY` | — | Telegram bot token |
| `APP__TELEGRAM_CHAT_IDS` | — | Chat IDs to notify |
| `APP__QUIET_MODE_ENABLED` | `false` | Suppress alerts during a window |
| `APP__QUIET_MODE_START_HOUR` / `END_HOUR` | — | Quiet window (24h hours) |

Config is read once at startup from the process env — set these before running.

## Data sources

- [CoinGecko](https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_vol=true) — price + 24h volume
- [bitdis.org](https://bitdis.org/api/live-data) — reachable Bitcoin nodes
- [alternative.me](https://api.alternative.me/fng/) — Fear & Greed index
- [colintalkscrypto](https://www.colintalkscrypto.com/cbbi/) — CBBI index

## Testing

Integration-style tests live in `tests/`:

- `tests/tests.rs` — declares `unit` and `helpers` modules
- `tests/unit/` — unit tests for price-change detection, quiet-mode, the cache,
  config defaults, and the notification formatter
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

Or via Docker (`Dockerfile` installs `procps` for the healthcheck and runs
tzdata non-interactively).
