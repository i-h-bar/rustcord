# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rustcord is a Discord bot for Magic: The Gathering card searching and guessing games. It uses fuzzy matching to handle misspellings and supports searching by card name, set, and artist.

**Key Features:**
- Card search with fuzzy matching (Jaro-Winkler algorithm with 0.75 threshold)
- Guessing game with difficulty levels (Easy: 8 guesses, Medium: 6 guesses, Hard: 4 guesses)
- Message parsing for inline card queries: `[[card name | set=code | artist=name]]`
- Discord slash commands: `/search`, `/play`, `/guess`, `/give_up`, `/help`

## Development Commands

### Building and Running
```bash
# Standard build
cargo build

# Run the bot (requires .env file with credentials)
cargo run

# Build for Raspberry Pi (ARM64)
make setup           # One-time: install cross-compilation target
make build_pi        # Cross-compile for aarch64
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks (Jaro-Winkler performance tests)
cargo bench
```

### Linting
```bash
# Standard lint (fmt + clippy)
make lint

# Pedantic lint (stricter clippy rules)
make mega_lint

# Individual commands
cargo fmt
cargo clippy
cargo clippy -- -W clippy::pedantic
```

## Architecture

This codebase follows **Hexagonal Architecture** (Ports and Adapters pattern) with clear separation of concerns:

### Three-Layer Structure

```
┌─────────────────────────────────────────┐
│          PORTS (Clients)                │  Discord bot interface
│  - Client trait                         │
│  - MessageInteraction trait             │
│  - GameInteraction trait                │
│  - Discord-specific implementations     │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│          DOMAIN (Business Logic)        │  Pure business logic
│  - App: Central coordinator             │
│  - Card search & fuzzy matching         │
│  - Game state management                │
│  - Query parsing (regex captures)       │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│       ADAPTERS (External Systems)       │  Infrastructure
│  - CardStore: Postgres queries          │
│  - Cache: Redis (game state)            │
│  - ImageStore: Filesystem               │
└─────────────────────────────────────────┘
```

### Key Architectural Patterns

**Dependency Injection via Generics:**
The `App` struct uses generic type parameters for all adapters, enabling easy testing with mocks:
```rust
App<IS, CS, C> where
    IS: ImageStore + Send + Sync,
    CS: CardStore + Send + Sync,
    C: Cache + Send + Sync
```

**Trait-Based Abstraction:**
All external systems are defined as traits in `src/adapters/*/mod.rs` with `#[cfg_attr(test, automock)]` for mockall integration. Concrete implementations are in subdirectories (e.g., `postgres/`, `redis/`, `file_system/`).

**Game State Persistence:**
Game state is serialized to RON format and stored in Redis with 86400s (24h) TTL. Per-channel mutex locks prevent race conditions during concurrent guesses.

### Critical Domain Logic

**Fuzzy Matching (`src/domain/utils/fuzzy.rs`):**
- Custom Jaro-Winkler implementation using bitmask optimization
- Threshold: 0.75 for both card search and guess validation
- `winkliest_match()`: finds best match from candidate list

**Query Parsing (`src/domain/query.rs`):**
- Regex pattern in `CARD_QUERY_RE` captures: `[[name | set=X | artist=Y]]`
- Set code vs set name: <5 chars = code, ≥5 chars = full name
- All inputs normalized via `normalise()`: NFKC Unicode → remove punctuation → lowercase

**Card Search Flow (`src/domain/search.rs`):**
1. Parse query → QueryParams
2. Determine search type (by set, artist, or distinct cards)
3. CardStore returns candidates from Postgres
4. Fuzzy match to find best candidate
5. ImageStore fetches images by UUID
6. Return (Card, Images) tuple

**Game State Machine (`src/domain/functions/game/state.rs`):**
- Difficulty determines max_guesses and reveal multiplier
- `add_guess()` increments counter, checked against max
- State serialized to RON, stored in Redis by channel ID
- Win: fuzzy match > 0.75, Loss: guesses ≥ max_guesses

## Environment Configuration

Required environment variables in `.env` (debug builds only):
```bash
BOT_TOKEN=<Discord bot token>
PSQL_URI=postgres://user:pass@host/database
REDIS_URL=redis://host/
IMAGES_DIR=<path to card images directory>
RUST_LOG=warn,rustcord=info  # Logging level
```

**Image Storage Structure:**
```
IMAGES_DIR/
  images/           # Full card images (UUID.png)
  illustrations/    # Cropped illustrations (UUID.png)
```

## Database Schema Notes

The Postgres schema uses GIN indexes for fuzzy text search (see commit history for optimization details). Key queries in `src/adapters/card_store/postgres/queries.rs`:
- `FUZZY_SEARCH_DISTINCT_CARDS`: Search across all cards
- `FUZZY_SEARCH_CARD_AND_SET_NAME`: Scoped to specific set
- `FUZZY_SEARCH_CARD_AND_ARTIST`: Scoped to artist
- `RANDOM_CARD` / `RANDOM_SET_CARD`: Game initialization

Card data includes front/back faces (for double-faced cards), illustration IDs, mana costs, power/toughness, and Scryfall URLs.

## Testing Strategy

See `TEST_COVERAGE_STRATEGY.md` for comprehensive testing plan. Current coverage is minimal (6 tests). Priority areas:
1. Game logic (state transitions, difficulty settings)
2. Fuzzy matching edge cases
3. Query parsing with regex captures
4. Card store integration tests

**Testing with Mocks:**
All adapter traits have mockall implementations. Example pattern:
```rust
let mut mock_store = MockCardStore::new();
mock_store.expect_search()
    .with(eq("normalized name"))
    .return_const(Some(vec![card]));
```

## Common Development Patterns

**Adding New Discord Commands:**
1. Define command in `src/ports/clients/discord/commands/register/`
2. Implement handler in `src/ports/clients/discord/commands/`
3. Add domain logic to `src/domain/functions/`
4. Register command in Discord client setup

**Adding New Adapter Methods:**
1. Add trait method to `src/adapters/*/mod.rs`
2. Implement in concrete adapter (postgres/redis/file_system)
3. Update mockall expectations in tests

**Card Data Access:**
Always use Card methods (`front_image_id()`, `illustration_ids()`, etc.) rather than direct field access for future-proofing.

## Deployment Notes

The bot is deployed to ARM64 architecture (Raspberry Pi). Use `make build_pi` for cross-compilation. The Docker setup (see `Dockerfile`) includes benchmark execution in the build process.

## Repository Information

- Issues: https://github.com/i-h-bar/rustcord/issues
- Discord Support: https://discord.gg/m9FjpPRAxt
- Bot Invite: https://discord.com/oauth2/authorize?client_id=1315969494161559595&permissions=277025507328&integration_type=0&scope=bot
