# Test Coverage Strategy - Rustcord

## Current State Analysis

### Test Coverage: Excellent Progress! ✅✅
- **Current:** 90 unit tests (15x increase from 6!)
- **Breakdown:**
  - ✅ 21 tests in fuzzy.rs (was 3)
  - ✅ 18 tests in state.rs (was 0)
  - ✅ 18 tests in query.rs (was 0)
  - ✅ 9 tests in search.rs (was 1)
  - ✅ 8 tests in guess.rs (was 0)
  - ✅ 8 tests in play.rs (was 0)
  - ✅ 5 tests in give_up.rs (was 0)
  - ✅ 2 tests in mutex.rs (existing)
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

#### 2. Game Logic (`src/domain/functions/game/`) ✅ DONE
- **Status:** ✅ 18 tests in `state.rs` (was 0)
- **Priority:** CRITICAL
- **Files:** `guess.rs`, `play.rs`, `state.rs`, `give_up.rs`
- **Why:** Core game mechanics; bugs directly impact user experience
- **Tests Completed:**
  - ✅ GameState creation and state transitions
  - ✅ Difficulty levels (Easy/Medium/Hard) boundary tests
  - ✅ Guess counting and max guess validation
  - ✅ Game state serialization/deserialization (RON format)
  - ✅ Cache integration (add, fetch, delete)
  - ✅ Invalid state handling
  - ⏳ Win condition detection in guess.rs (TODO)
  - ⏳ Concurrent game handling per channel in guess.rs (TODO)
  - ⏳ play.rs command logic (TODO)
  - ⏳ give_up.rs command logic (TODO)

#### 3. Card Search & Query Logic (`src/domain/search.rs`, `src/domain/query.rs`) ✅ DONE
- **Status:** ✅ 18 tests in query.rs + 1 in search.rs (was 1 total)
- **Priority:** CRITICAL
- **Why:** Primary feature - users search cards constantly
- **Tests Completed:**
  - ✅ `QueryParams::from()` regex capture parsing
  - ✅ Set code vs set name distinction (<5 chars)
  - ✅ Artist search validation
  - ✅ Message parsing with multiple card references
  - ✅ Whitespace handling bug fixed
  - ✅ Punctuation and case normalization
  - ⏳ Cache hit/miss scenarios in search.rs (TODO)
  - ⏳ Error handling for no matches (TODO)
  - ⏳ parse_message with multiple cards (TODO)

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
All critical business logic now has comprehensive test coverage (90 tests total).

### Current Priorities - Move to Tier 2:
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

---

*Generated: 2025-10-11*
*Last Updated: 2025-10-13 (Phase 2 complete, 90 tests passing)*
