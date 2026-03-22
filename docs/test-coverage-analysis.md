# Test Coverage Analysis

## Current State

The codebase contains **11 Rust crates** with a total of **102 source files**. Of these, only **20 files (19.6%)** contain tests, accounting for **53 test functions** (23 synchronous, 30 async via `#[tokio::test]`). There is no code coverage tooling configured — no `tarpaulin`, `llvm-cov`, or CI coverage gate.

### Coverage by Crate

| Crate | Files | Files w/ Tests | % Covered | Notes |
|---|---|---|---|---|
| `koad-sandbox` | 2 | 2 | **100%** | Best-tested crate |
| `koad-plugins` | 2 | 2 | **100%** | Well-covered |
| `koad-codegraph` | 1 | 1 | **100%** | Well-covered |
| `koad-citadel` | 25 | 6 | **24%** | Core daemon, partially tested |
| `koad-core` | 18 | 5 | **28%** | Shared primitives, mostly untested |
| `koad-cass` | 11 | 2 | **18%** | Memory/AI services, mostly untested |
| `koad-intelligence` | 6 | 1 | **17%** | Inference router, mostly untested |
| `koad-cli` | 24 | 1 | **4%** | CLI layer, nearly untested |
| `koad-board` | 6 | 0 | **0%** | Project board sync, zero tests |
| `koad-bridge-notion` | 4 | 0 | **0%** | Notion bridge, zero tests |
| `koad-proto` | 1 | 0 | **0%** | Protobuf glue, zero tests |

---

## Priority Areas for Improvement

The following areas are ranked by risk (criticality × lack of coverage).

---

### 1. `koad-core`: Session & Distributed Locking

**Files:** `session.rs`, `lock.rs`, `pid.rs`
**Priority:** HIGH — foundational primitives used by every other crate

#### `session.rs` — Agent session lifecycle

`AgentSession` tracks the live state of agents: identity, environment, project boundaries, heartbeat timestamps, and hot context chunks. The `is_active(timeout_secs)` method is used across the daemon to determine whether a session should be reaped.

**Tests to write:**
```rust
#[test]
fn test_session_is_active_within_window() {
    let session = AgentSession::new(identity, env, body_id);
    assert!(session.is_active(60)); // just created, should be active
}

#[test]
fn test_session_is_inactive_after_timeout() {
    let mut session = AgentSession::new(identity, env, body_id);
    session.last_heartbeat = Utc::now() - Duration::seconds(120);
    assert!(!session.is_active(60));
}

#[test]
fn test_session_ids_are_unique() {
    let s1 = AgentSession::new(...);
    let s2 = AgentSession::new(...);
    assert_ne!(s1.session_id, s2.session_id);
}
```

#### `lock.rs` — Distributed locking (RAII guard over Redis)

`SectorLockGuard` is the main concurrency primitive preventing two agents from writing to the same sector simultaneously. Lock acquisition failure is a valid, expected path — it must be tested.

**Tests to write:**
```rust
#[tokio::test]
async fn test_lock_acquired_and_released() { ... }

#[tokio::test]
async fn test_second_acquire_fails_while_held() { ... }

#[tokio::test]
async fn test_lock_released_on_guard_drop() { ... }

#[tokio::test]
async fn test_cannot_release_lock_owned_by_other() { ... }
```

#### `pid.rs` — Single-instance enforcement

`PidGuard` ensures only one daemon runs at a time. Without tests, bugs here would allow two competing daemons to corrupt shared state.

**Tests to write:**
```rust
#[test]
fn test_pid_file_created_on_guard_new() { ... }

#[test]
fn test_pid_file_removed_on_drop() { ... }

#[test]
fn test_stale_pid_file_overwritten() { ... }  // crashed daemon
```

---

### 2. `koad-citadel`: Storage Bridge (L1/L2 CQRS)

**File:** `state/storage_bridge.rs`
**Priority:** HIGH — all persistent state flows through this

`CitadelStorageBridge` implements a Redis (L1) + SQLite (L2) dual-store with a periodic drain loop. Cache misses fall through to L2 and repopulate L1. This logic is subtle and failure here causes data loss or stale reads.

**Tests to write:**
```rust
#[tokio::test]
async fn test_l1_write_and_read() {
    // write to L1, read back immediately — should hit cache
}

#[tokio::test]
async fn test_l2_fallback_on_l1_miss() {
    // populate L2 directly, flush L1, read → should repopulate L1
}

#[tokio::test]
async fn test_drain_moves_l1_to_l2() {
    // write to L1, run drain, verify L2 has entry with timestamp
}

#[tokio::test]
async fn test_hydrate_repopulates_l1_from_l2() {
    // start with empty Redis, hydrate from SQLite, read from L1
}

#[tokio::test]
async fn test_permission_enforcement_on_set_state() {
    // caller_tier below required level → should be rejected
}
```

---

### 3. `koad-citadel`: Workspace Manager (Git Worktrees)

**File:** `workspace/manager.rs`
**Priority:** HIGH — agent task isolation depends on this

`WorkspaceManager` creates isolated Git worktrees per agent/task. A bug in branch naming, path construction, or git command error handling would silently break agent task isolation.

**Tests to write:**
```rust
#[tokio::test]
async fn test_worktree_branch_name_is_lowercased() {
    // agent "MyAgent", task "T1" → branch "myagent/T1"
}

#[tokio::test]
async fn test_worktree_path_constructed_correctly() {
    // base_path/agent_name/task_id
}

#[tokio::test]
async fn test_remove_worktree_is_idempotent() {
    // removing a non-existent worktree logs a warning but doesn't panic
}

// Integration test gated on git availability:
#[tokio::test]
#[ignore = "requires git repo"]
async fn test_create_and_remove_worktree_integration() { ... }
```

---

### 4. `koad-intelligence`: Inference Router & Ollama Client

**Files:** `router.rs`, `clients/ollama.rs`
**Priority:** MEDIUM-HIGH — all AI inference calls pass through here

The `InferenceRouter` currently always routes to the local Ollama client. The `OllamaClient` parses a float from the LLM response for significance scoring — a fragile operation that needs a fallback and test coverage.

**Tests to write (router):**
```rust
#[tokio::test]
async fn test_all_task_variants_route_to_local() {
    for task in [Distillation, Evaluation, Reasoning] {
        let client = router.select(task);
        // verify it's the local client
    }
}
```

**Tests to write (Ollama client) — use `wiremock` for HTTP mocking:**
```rust
#[tokio::test]
async fn test_chat_parses_response_field() {
    // mock POST /api/generate → {"response": "hello"}
    // assert chat("...") == Ok("hello")
}

#[tokio::test]
async fn test_score_falls_back_to_0_5_on_invalid_float() {
    // mock returns "not a number" → score should be 0.5
}

#[tokio::test]
async fn test_chat_returns_error_on_http_500() { ... }

#[tokio::test]
async fn test_connect_timeout_respected() { ... }
```

---

### 5. `koad-cass`: Memory & Symbol Services

**Files:** `services/memory.rs`, `services/symbol.rs`
**Priority:** MEDIUM — gRPC services with background tasks

`CassMemoryService::record_episode` spawns a background tokio task to score episode significance. If the task panics silently, episodes go unscored without any observable error. `CassSymbolService` spawns a blocking thread for indexing — same concern.

**Tests to write (memory):**
```rust
#[tokio::test]
async fn test_commit_fact_roundtrip() {
    // commit, then query by domain → fact returned
}

#[tokio::test]
async fn test_record_episode_spawns_scoring() {
    // verify background task is spawned and scoring is called
    // use mock IntelligenceRouter that records invocations
}

#[tokio::test]
async fn test_query_facts_respects_limit() {
    // commit 10 facts, query with limit=3 → 3 returned
}
```

---

### 6. `koad-board`: BoardSyncer (GitHub → Project Board Sync)

**File:** `sync.rs`
**Priority:** MEDIUM — external mutations, dry-run mode must be verified

`BoardSyncer::run()` mutates a GitHub project board. A bug in dry-run detection would push unwanted mutations to production. Task weight extraction from issue bodies uses regex — this should be unit-tested against real-world markdown patterns.

**Tests to write:**
```rust
#[test]
fn test_dry_run_prevents_mutations() {
    let syncer = BoardSyncer::new(mock_client, project, dry_run: true);
    syncer.run().await;
    mock_client.assert_no_mutations_called();
}

#[test]
fn test_task_weight_extraction_trivial() {
    let body = "## Weight\n- [x] Trivial";
    assert_eq!(extract_weight(body), Some(Weight::Trivial));
}

#[test]
fn test_task_weight_extraction_complex() { ... }

#[test]
fn test_task_weight_missing_returns_none() { ... }

#[test]
fn test_duplicate_issues_not_re_added() { ... }
```

---

### 7. `koad-bridge-notion`: Markdown Parser

**File:** `parser.rs`
**Priority:** MEDIUM-LOW — pure transformation logic, easy to test, currently at 0%

`parse_blocks_to_markdown` is a pure function with no I/O dependencies. This is the cheapest category of tests to write with the highest reliability payoff. Every block type is a distinct code path.

**Tests to write:**
```rust
#[test]
fn test_heading_1_renders_with_hash() {
    let block = heading_block("h1", "Title");
    assert_eq!(parse_blocks_to_markdown(vec![block]), "# Title\n");
}

#[test]
fn test_todo_checked() {
    let block = todo_block("Task", checked: true);
    assert_eq!(parse_blocks_to_markdown(vec![block]), "- [x] Task\n");
}

#[test]
fn test_numbered_list_uses_number_prefix() { ... }

#[test]
fn test_code_block_includes_language_tag() { ... }

#[test]
fn test_unknown_block_type_is_skipped_not_panicked() { ... }
```

---

### 8. `koad-core`: Token Counter

**File:** `tokens.rs`
**Priority:** LOW — simple utility with a known-good upstream library, but fallback path is untested

**Tests to write:**
```rust
#[test]
fn test_known_token_count() {
    // "Hello, world!" → known token count for cl100k_base
    assert_eq!(count_tokens("Hello, world!"), 4);
}

#[test]
fn test_empty_string_returns_zero() {
    assert_eq!(count_tokens(""), 0);
}
```

---

## Infrastructure Recommendations

### Add Code Coverage Tooling

Add `cargo-tarpaulin` or `cargo-llvm-cov` and run it in CI:

```toml
# .github/workflows/test.yml
- name: Install tarpaulin
  run: cargo install cargo-tarpaulin

- name: Run tests with coverage
  run: cargo tarpaulin --out Xml --workspace

- name: Upload coverage
  uses: codecov/codecov-action@v4
```

### Add a `test.yml` Workflow

Currently neither `auto-project.yml` nor `compliance.yml` runs `cargo test`. A basic workflow:

```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
```

### Consistent Test Patterns

The best-tested crates (`koad-sandbox`, `koad-citadel` quota/monitor) share these patterns worth adopting everywhere:

- **Factory helpers** (`make_redis()`, `make_validator()`) — reduce boilerplate, enforce consistent setup
- **`tempfile::tempdir()`** — isolate filesystem/database state per test
- **Feature-gated integration tests** — `#[cfg(feature = "integration")]` or env-var guards like `KOAD_TEST_DOCKER=1`
- **Async tests** via `#[tokio::test]` for all async code paths

### Add `wiremock` for HTTP Testing

`koad-intelligence` and `koad-bridge-notion` make HTTP calls. Add `wiremock` as a dev-dependency for reliable HTTP mocking without spinning up real servers:

```toml
# crates/koad-intelligence/Cargo.toml
[dev-dependencies]
wiremock = "0.6"
```

---

## Summary

| Area | Files | Current Tests | Recommendation |
|---|---|---|---|
| Core session/lock/pid | 3 | 0 | Write 10–15 unit tests |
| Citadel storage bridge | 1 | 0 | Write 5–8 async integration tests |
| Citadel workspace manager | 1 | 0 | Write 3–5 tests + 1 integration test |
| Intelligence router + Ollama | 2 | 1 (router only) | Add HTTP mock tests for Ollama |
| CASS memory/symbol services | 2 | 0 | Write 8–10 gRPC unit tests |
| Board syncer | 1 | 0 | Write 5–7 tests incl. dry-run |
| Notion parser | 1 | 0 | Write 8–10 pure unit tests |
| Token counter | 1 | 0 | Write 2–3 unit tests |
| **CI / coverage tooling** | — | — | Add `test.yml` + tarpaulin |
