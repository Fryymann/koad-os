# Deterministic Kernel Shutdown Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the Citadel Kernel to use `tokio::task::JoinSet` for tracking background tasks, ensuring a deterministic and graceful shutdown by waiting for all tasks to complete instead of using a hardcoded sleep.

**Architecture:**
1. Update `Kernel` struct to include a `JoinSet<()>`.
2. Modify `KernelBuilder::start` to spawn background tasks into the `JoinSet`.
3. Update `Kernel::shutdown` to await all tasks in the `JoinSet` after signaling shutdown.

**Tech Stack:** Rust, Tokio, anyhow

---

### Task 1: Update Kernel Struct and Imports

**Files:**
- Modify: `crates/koad-citadel/src/kernel.rs`

- [ ] **Step 1: Add JoinSet import**
Add `use tokio::task::JoinSet;` to the imports.

- [ ] **Step 2: Update Kernel struct**
Add `tasks: JoinSet<()>` to the `Kernel` struct.

### Task 2: Refactor KernelBuilder::start to use JoinSet

**Files:**
- Modify: `crates/koad-citadel/src/kernel.rs`

- [ ] **Step 1: Initialize JoinSet**
At the beginning of `KernelBuilder::start`, initialize `let mut tasks = JoinSet::new();`.

- [ ] **Step 2: Spawn background tasks into JoinSet**
Replace all `tokio::spawn(async move { ... })` calls with `tasks.spawn(async move { ... })`.
This includes:
  - Drain loop
  - Reaper loop
  - TCP listener
  - Admin UDS listener

- [ ] **Step 3: Update Kernel instantiation**
Pass the `tasks` JoinSet to the `Kernel` constructor at the end of `start`.

### Task 3: Update Kernel::shutdown to await tasks

**Files:**
- Modify: `crates/koad-citadel/src/kernel.rs`

- [ ] **Step 1: Change shutdown signature**
Change `pub async fn shutdown(self)` to `pub async fn shutdown(mut self)`.

- [ ] **Step 2: Replace sleep with JoinSet await**
Remove `tokio::time::sleep(Duration::from_millis(800)).await;`.
Add a loop to join all tasks:
```rust
        // 2. Wait for background tasks to settle
        while let Some(res) = self.tasks.join_next().await {
            if let Err(e) = res {
                error!("Kernel: Task join error: {}", e);
            }
        }
```

### Task 4: Verification

- [ ] **Step 1: Verify build**
Run `cargo check -p koad-citadel` in `/home/ideans/koados-citadel`.
