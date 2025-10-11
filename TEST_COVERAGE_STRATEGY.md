# Test Coverage Strategy - Rustcord

## Current State Analysis

### Test Coverage: Currently Minimal
- Only 6 unit tests total (3 in fuzzy.rs, 1 in search.rs, 2 tests per lib)
- ~2,835 lines of code with extremely low test coverage
- Good foundation: mockall already integrated for mocking

### Architecture: Clean Hexagonal/Ports-and-Adapters Design
- **Domain Layer:** Business logic (card search, game logic, fuzzy matching)
- **Adapters:** External integrations (Postgres, Redis, FileSystem)
- **Ports:** Client interfaces (Discord bot)

---

## Test Coverage Strategy (Prioritized)

### TIER 1: Critical Business Logic (Highest Priority)

These systems are core to the application's value and correctness:

#### 1. Fuzzy Matching System (`src/domain/utils/fuzzy.rs`)
- **Status:** ✓ Has tests (3 tests)
- **Action:** Expand edge cases
- **Priority:** HIGH
- **Why:** Powers all card search functionality; incorrect matches = poor UX
- **Additional Tests Needed:**
  - Empty strings
  - Unicode/special characters
  - Very long strings (>64 chars)
  - Performance benchmarks for large datasets

#### 2. Game Logic (`src/domain/functions/game/`)
- **Status:** ✗ No tests
- **Priority:** CRITICAL
- **Files:** `guess.rs`, `play.rs`, `state.rs`, `give_up.rs`
- **Why:** Core game mechanics; bugs directly impact user experience
- **Tests Needed:**
  - GameState creation and state transitions
  - Difficulty levels (Easy/Medium/Hard) boundary tests
  - Guess counting and max guess validation
  - Win condition detection (fuzzy match threshold 0.75)
  - Game state serialization/deserialization (RON format)
  - Concurrent game handling per channel

#### 3. Card Search & Query Logic (`src/domain/search.rs`, `src/domain/query.rs`)
- **Status:** Partial (1 test in search.rs)
- **Priority:** CRITICAL
- **Why:** Primary feature - users search cards constantly
- **Tests Needed:**
  - `QueryParams::from()` regex capture parsing
  - Set code vs set name distinction (<5 chars)
  - Artist search validation
  - Message parsing with multiple card references
  - Cache hit/miss scenarios
  - Error handling for no matches

---

### TIER 2: Data Layer Integration (High Priority)

#### 4. Card Store Adapter (`src/adapters/card_store/postgres/`)
- **Status:** ✗ No tests
- **Priority:** HIGH
- **Why:** Direct database queries; SQL errors can crash features
- **Tests Needed:**
  - Integration tests with test database (use `sqlx::test`)
  - Query validation (fuzzy search, artist search, set search)
  - Random card selection logic
  - Set abbreviation lookup
  - Error handling for malformed queries
  - Connection pool behavior

#### 5. Cache Layer (`src/adapters/cache/redis.rs`)
- **Status:** ✗ No tests
- **Priority:** HIGH
- **Why:** Game state persistence; failures cause lost games
- **Tests Needed:**
  - Mock Redis for unit tests
  - TTL validation (86400 seconds)
  - Connection error handling
  - Key collision scenarios
  - Game state corruption recovery

#### 6. Image Store (`src/adapters/image_store/file_system.rs`)
- **Status:** ✗ No tests
- **Priority:** MEDIUM
- **Why:** Missing images degrade UX but don't break functionality
- **Tests Needed:**
  - File not found error handling
  - Front/back image loading
  - Illustration vs full image distinction
  - Path construction validation

---

### TIER 3: Utility & Support Functions (Medium Priority)

#### 7. Normalization Utils (`src/domain/utils/mod.rs`)
- **Status:** Unknown (need to verify)
- **Priority:** MEDIUM
- **Why:** Used everywhere; bugs cascade through system
- **Tests Needed:**
  - Unicode normalization edge cases
  - Special character handling
  - Case normalization

#### 8. Mutex Utilities (`src/domain/utils/mutex.rs`)
- **Status:** ✗ No tests
- **Priority:** MEDIUM
- **Why:** Prevents race conditions in game state
- **Tests Needed:**
  - Concurrent access simulation
  - Deadlock prevention
  - Lock cleanup

---

### TIER 4: Client Layer (Lower Priority)

#### 9. Discord Integration (`src/ports/clients/discord/`)
- **Status:** ✗ No tests
- **Priority:** LOW-MEDIUM
- **Why:** Already abstracted behind traits; test via mocks
- **Tests Needed:**
  - Command registration validation
  - Embed creation logic
  - Message parsing (bracket syntax `[[card]]`)
  - Error message handling
  - Interaction response validation

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
1. **Expand fuzzy matching tests** - Build confidence in core algorithm
2. **Game state tests** - Cover all state transitions
3. **Query parsing tests** - Validate regex captures

### Phase 2: Core Logic (Week 3-4)
4. **Game command tests** (play, guess, give_up)
5. **Search functionality tests**
6. **Card store integration tests** (with test DB)

### Phase 3: Infrastructure (Week 5-6)
7. **Cache layer tests** (mock Redis)
8. **Image store tests** (mock filesystem)
9. **Error handling paths**

### Phase 4: Polish (Week 7+)
10. **Integration tests** (end-to-end workflows)
11. **Performance tests** (fuzzy matching at scale)
12. **Discord client tests** (command handling)

---

## Recommended Testing Tools

### Add to dev-dependencies:
```toml
[dev-dependencies]
mockall = "0.13.1"      # ✓ Already included
criterion = "0.7"        # ✓ Already included
tokio-test = "0.4"       # Async test utilities
rstest = "0.18"          # Parameterized tests
proptest = "1.4"         # Property-based testing
fake = "2.9"             # Test data generation
```

### For integration tests:
- `sqlx::test` macro for database tests
- `testcontainers` for Redis/Postgres containers

### Coverage measurement:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# View coverage
open coverage/index.html
```

---

## Success Metrics

- **Immediate goal:** 40-50% coverage (focus on business logic)
- **3-month goal:** 70% coverage (include adapters)
- **Ideal state:** 80%+ coverage (comprehensive)

### Critical systems to hit 90%+ first:
- Fuzzy matching
- Game logic
- Query parsing
- Search functionality

---

## Quick Start

### Run existing tests:
```bash
cargo test
```

### Run with output:
```bash
cargo test -- --nocapture
```

### Run specific test:
```bash
cargo test test_jaro_winkler_bitmasl
```

### Generate coverage:
```bash
cargo tarpaulin --out Html
```

---

## Next Steps

1. Start with **Tier 1 - Game Logic** (zero coverage, critical functionality)
2. Add parametrized tests for **Query Parsing**
3. Create integration tests for **Card Store** with test database
4. Implement property-based tests for **Fuzzy Matching**
5. Set up CI/CD pipeline to track coverage over time

---

*Generated: 2025-10-11*
