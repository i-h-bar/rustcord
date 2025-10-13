# Test Coverage Strategy - Rustcord

## Current State Analysis

### Test Coverage: Outstanding Progress! ✅✅✅
- **Current:** 120 unit tests (20x increase from 6!)
- **Breakdown:**
  - ✅ 24 tests in utils/mod.rs normalise() (was 0)
  - ✅ 21 tests in fuzzy.rs (was 3)
  - ✅ 18 tests in state.rs (was 0)
  - ✅ 18 tests in query.rs (was 0)
  - ✅ 9 tests in search.rs (was 1)
  - ✅ 8 tests in guess.rs (was 0)
  - ✅ 8 tests in play.rs (was 0)
  - ✅ 8 tests in mutex.rs (was 2) **EXPANDED!**
  - ✅ 5 tests in give_up.rs (was 0)
  - ✅ 1 test in other modules
- ~2,835 lines of code
- Good foundation: mockall already integrated for mocking

### Architecture: Clean Hexagonal/Ports-and-Adapters Design
- **Domain Layer:** Business logic (card search, game logic, fuzzy matching)
- **Adapters:** External integrations (Postgres, Redis, FileSystem)
- **Ports:** Client interfaces (Discord bot)

---

## Test Coverage Strategy (Prioritized)

### TIER 1: Critical Business Logic (Highest Priority) ✅ COMPLETED

These systems are core to the application's value and correctness:

#### 1. Fuzzy Matching System (`src/domain/utils/fuzzy.rs`) ✅ DONE
- **Status:** ✅ 21 tests (was 3)
- **Action:** ~~Expand edge cases~~ COMPLETED
- **Priority:** HIGH
- **Why:** Powers all card search functionality; incorrect matches = poor UX
- **Tests Added:**
  - ✅ Empty strings
  - ✅ Unicode/special characters
  - ✅ Very long strings (>64 chars)
  - ✅ Threshold boundary testing (0.75)
  - ✅ Case sensitivity verification
  - ✅ Prefix boost validation
  - ✅ Transposition handling
  - ✅ Real card name scenarios with typos
  - ⏳ Performance benchmarks for large datasets (TODO - use Criterion)

#### 2. Game Logic (`src/domain/functions/game/`) ✅ COMPLETE
- **Status:** ✅ 39 tests total (18 state + 8 guess + 8 play + 5 give_up)
- **Priority:** CRITICAL
- **Files:** `guess.rs`, `play.rs`, `state.rs`, `give_up.rs`
- **Why:** Core game mechanics; bugs directly impact user experience
- **Tests Completed:**
  - ✅ GameState creation and state transitions (state.rs)
  - ✅ Difficulty levels (Easy/Medium/Hard) boundary tests (state.rs)
  - ✅ Guess counting and max guess validation (state.rs)
  - ✅ Game state serialization/deserialization RON format (state.rs)
  - ✅ Cache integration - add, fetch, delete (state.rs)
  - ✅ Invalid state handling (state.rs)
  - ✅ Win condition detection with fuzzy matching (guess.rs)
  - ✅ Loss condition at max guesses (guess.rs)
  - ✅ Concurrent game handling via mutex locks (mutex.rs)
  - ✅ Play command initialization and validation (play.rs)
  - ✅ Set validation - abbreviations and full names (play.rs)
  - ✅ Give up command and state cleanup (give_up.rs)

#### 3. Card Search & Query Logic (`src/domain/search.rs`, `src/domain/query.rs`) ✅ COMPLETE
- **Status:** ✅ 27 tests total (18 query + 9 search)
- **Priority:** CRITICAL
- **Why:** Primary feature - users search cards constantly
- **Tests Completed:**
  - ✅ `QueryParams::from()` regex capture parsing (query.rs)
  - ✅ Set code vs set name distinction (<5 chars) (query.rs)
  - ✅ Artist search validation (query.rs)
  - ✅ Message parsing with multiple card references (query.rs)
  - ✅ Whitespace handling bug fixed (query.rs)
  - ✅ Punctuation and case normalization (query.rs)
  - ✅ Error handling for card not found (search.rs)
  - ✅ Search with set codes and set names (search.rs)
  - ✅ Search with artist filtering (search.rs)
  - ✅ parse_message with single/multiple/no cards (search.rs)
  - ✅ find_card with all query types (search.rs)
  - ✅ Fuzzy matching for set names (search.rs)

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

#### 7. Normalization Utils (`src/domain/utils/mod.rs`) ✅ COMPLETE
- **Status:** ✅ 24 tests for `normalise()` function
- **Priority:** MEDIUM
- **Why:** Used everywhere; bugs cascade through system
- **Tests Completed:**
  - ✅ Unicode normalization (NFKC) edge cases
  - ✅ Special character handling (™, é, æ, Japanese)
  - ✅ Case normalization
  - ✅ Punctuation removal (commas, apostrophes, brackets, parentheses)
  - ✅ Hyphen to space conversion
  - ✅ Real MTG card name validation
  - ✅ Empty strings and edge cases
  - ✅ Idempotency verification
  - ✅ **CRITICAL BUG FOUND AND FIXED**: `.replace()` → `.replace_all()`

#### 8. Mutex Utilities (`src/domain/utils/mutex.rs`) ✅ COMPLETE
- **Status:** ✅ 8 tests (was 2) **EXPANDED!**
- **Priority:** MEDIUM
- **Why:** Prevents race conditions in game state
- **Tests Completed:**
  - ✅ Lock acquisition by channel ID
  - ✅ Lock retrieval by name
  - ✅ Reference counting increment
  - ✅ Multiple acquisitions of same lock name
  - ✅ Lock isolation between different names
  - ✅ Concurrent access serialization (same lock blocks)
  - ✅ Parallel execution for different locks (no blocking)
  - ✅ Guard release on drop
  - Note: Async drop cleanup tests omitted due to timing issues with async-dropper library

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

### Phase 1: Foundation (Week 1-2) ✅ COMPLETED
1. ✅ **Expand fuzzy matching tests** - Build confidence in core algorithm
2. ✅ **Game state tests** - Cover all state transitions
3. ✅ **Query parsing tests** - Validate regex captures

### Phase 2: Core Logic (Week 3-4) ✅ COMPLETED
4. ✅ **Game command tests** (play, guess, give_up) - ALL DONE
   - ✅ guess.rs - 8 tests covering win/loss conditions, fuzzy matching, edge cases
   - ✅ play.rs - 8 tests covering set validation, difficulties, abbreviations
   - ✅ give_up.rs - 5 tests covering game termination, state deletion
5. ✅ **Search functionality tests** - 9 tests in search.rs
   - ✅ parse_message with single/multiple/no cards
   - ✅ find_card with set codes, set names, and artists
   - ✅ Error handling for card not found
   - ✅ Fuzzy matching for set names
6. ⏳ **Card store integration tests** (with test DB) - TODO

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

## Next Steps (Pick up here in next session!)

### Tier 1: COMPLETE! ✅
All critical business logic now has comprehensive test coverage:
- ✅ Fuzzy matching (21 tests)
- ✅ Game logic - all commands (39 tests: state, guess, play, give_up)
- ✅ Card search & query (27 tests)

### Tier 3: COMPLETE! ✅
Utility functions fully tested:
- ✅ Normalization utils (24 tests) + **CRITICAL BUG FIXED**
- ✅ Mutex utilities (8 tests - 4x expansion)

**Total: 120 tests (20x increase from 6)**

### Current Priorities - Tier 2 Infrastructure:
2. ⏳ **Card Store Integration Tests** (src/adapters/card_store/postgres/)
   - Use `sqlx::test` macro for database tests
   - Mock database queries
   - Test error handling

3. ⏳ **Cache Layer Tests** (src/adapters/cache/redis.rs)
   - Mock Redis operations
   - Test TTL behavior
   - Error scenarios

4. ⏳ **Image Store Tests** (src/adapters/image_store/file_system.rs)
   - Mock filesystem operations
   - Test missing file handling

### Future:
5. Set up CI/CD pipeline to track coverage over time
6. Add property-based tests for Fuzzy Matching with `proptest`
7. Performance benchmarks using existing Criterion setup

---

## Bug Fixes Made
- ✅ Fixed regex whitespace handling in `CARD_QUERY_RE` (changed `(?:\s)?` to `(?:\s)*`)
- ✅ Added `.trim()` to QueryParams parsing to handle extra spaces
- ✅ **CRITICAL BUG FIX**: Fixed `normalise()` function in src/domain/utils/mod.rs:36
  - Changed `.replace()` to `.replace_all()`
  - Was only removing FIRST punctuation character, now removes all
  - Affected all card name normalization throughout the application
  - Tests revealed: "Card (Name)" became "card name)" instead of "card name"

---

*Generated: 2025-10-11*
*Last Updated: 2025-10-13 (Tier 1 & 3 COMPLETE, 120 tests passing, 1 critical bug fixed)*
