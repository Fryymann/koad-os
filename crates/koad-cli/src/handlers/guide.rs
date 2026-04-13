//! # KoadOS Field Guide & Protocol Handler
//! 
//! Provides agents with immediate access to the KoadOS Canon, Prime Directives, 
//! and standard operating procedures. This is the source of truth for 
//! "How to be a KoadOS Agent" at any station or outpost.

use anyhow::Result;
use koad_core::config::KoadConfig;

/// Entry point for `koad intel guide` (aliased to `koad guide` for agents).
pub async fn handle_guide_action(topic: Option<String>, _config: &KoadConfig) -> Result<()> {
    match topic.as_deref() {
        Some("quick") | Some("start") | None => show_quick_start(),
        Some("canon") | Some("directives") => show_prime_directives(),
        Some("workflow") | Some("cycle") => show_standard_workflow(),
        Some("ais") | Some("efficiency") => show_ais_efficiency(),
        Some("xp") | Some("saveup") => show_xp_system(),
        Some("worktree") | Some("parallel") => show_worktree_conventions(),
        Some(t) => {
            println!("\x1b[31m[ERROR]\x1b[0m Unknown guide topic: '{}'", t);
            println!("Available topics: quick, canon, workflow, ais, xp, worktree");
        }
    }
    Ok(())
}

fn show_quick_start() {
    println!("\x1b[1;34m--- KoadOS Agent Quick Start ---\x1b[0m");
    println!("1. \x1b[1mHydrate:\x1b[0m Run `agent-boot <name>` to anchor your identity.");
    println!("2. \x1b[1mOrient:\x1b[0m Read your Context Packet (CASS) and use `koad map look`.");
    println!("3. \x1b[1mDiscovery:\x1b[0m \x1b[1mUse code-review-graph MCP tools FIRST.\x1b[0m Find symbols, callers, and impact.");
    println!("4. \x1b[1mResearch:\x1b[0m Fall back to `grep_search` and API maps. Never read full files > 50 lines.");
    println!("5. \x1b[1mStrategy:\x1b[0m Formulate a plan. Use `enter_plan_mode` for Medium+ tasks.");
    println!("6. \x1b[1mExecution:\x1b[0m Surgical updates only. Follow the Research -> Strategy -> Execution cycle.");
    println!("7. \x1b[1mValidate:\x1b[0m Run tests and linters. No change is complete without verification.");
    println!("\n\x1b[33mTip:\x1b[0m Use `koad guide canon` for the non-negotiable laws.");
}

fn show_prime_directives() {
    println!("\x1b[1;31m--- The KoadOS Prime Directives (Non-Negotiable) ---\x1b[0m");
    println!("1. \x1b[1mOne Body, One Ghost:\x1b[0m One agent per session in their own worktree.");
    println!("2. \x1b[1mThe Sanctuary Rule:\x1b[0m No unauthorized cross-directory modifications.");
    println!("3. \x1b[1mPlan Mode Law:\x1b[0m Mandatory for all Medium complexity tasks or higher.");
    println!("4. \x1b[1mGraph-First Discovery:\x1b[0m Use the Dynamic System Map before raw file scanning.");
    println!("5. \x1b[1mNo-Read Rule:\x1b[0m Forbidden from reading entire files over 50 lines. Use surgical extraction.");
    println!("6. \x1b[1mDood Approval:\x1b[0m Major architectural changes require human (Ian) oversight.");
    println!("7. \x1b[1mSecure Cognition:\x1b[0m Zero tolerance for secret leakage. Use the Vault.");
}

fn show_standard_workflow() {
    println!("\x1b[1;32m--- Standard Engineering Cycle ---\x1b[0m");
    println!("\x1b[1m[1] Discovery:\x1b[0m Use `code-review-graph` to map dependencies and impact.");
    println!("\x1b[1m[2] Research:\x1b[0m Validate graph findings, reproduce bugs, and explore symbols.");
    println!("\x1b[1m[3] Strategy:\x1b[0m Draft a design/plan. Share summary for approval.");
    println!("\x1b[1m[4] Execution:\x1b[0m Surgical Plan -> Act -> Validate loop per sub-task.");
    println!("\x1b[1m[5] Verification:\x1b[0m Exhaustive tests, linting, and type-checking.");
    println!("\x1b[1m[6] Documentation:\x1b[0m Update logs (SESSIONS_LOG, WORKING_MEMORY, Updates).");
}

fn show_ais_efficiency() {
    println!("\x1b[1;35m--- AIS: Agent Information System Efficiency ---\x1b[0m");
    println!("- \x1b[1mToken Budgeting:\x1b[0m Every token saved is compute for reasoning.");
    println!("- \x1b[1mGraph-First:\x1b[0m `code-review-graph` is cheaper and faster than Grep/Glob.");
    println!("- \x1b[1mContext Packets:\x1b[0m Rely on CASS summaries before raw file reads.");
    println!("- \x1b[1mSurgical Tools:\x1b[0m Use `grep_search` and `read_file` with line ranges.");
    println!("- \x1b[1mCognitive Offloading:\x1b[0m Let the Citadel handle file discovery and routing.");
}

fn show_xp_system() {
    println!("\x1b[1;33m--- The Experience Point (XP) System ---\x1b[0m");
    println!("- \x1b[1mEarn XP:\x1b[0m Clean KSRP exits, Saveup passes, and Gate discipline (+5 to +45 XP).");
    println!("- \x1b[1mPenalties:\x1b[0m Gate violations, destructive changes, or skipped PSRP passes (-5 to -25 XP).");
    println!("- \x1b[1mSaveup Protocol:\x1b[0m Every task requires a reflection (Fact, Learn, Ponder).");
    println!("- \x1b[1mXP Ledger:\x1b[0m The `XP_LEDGER.md` in your vault is the source of truth.");
}

fn show_worktree_conventions() {
    println!("\x1b[1;36m--- Worktree & Parallel Execution ---\x1b[0m");
    println!("- \x1b[1mGhost Worktrees:\x1b[0m Work in your assigned worktree (e.g., `~/koad-tyr/`).");
    println!("- \x1b[1mBranch Management:\x1b[0m Sovereignty over your own identity branch.");
    println!("- \x1b[1mSynchronization:\x1b[0m Pull from `nightly` daily; push via PR.");
    println!("- \x1b[1mTask Manifests:\x1b[0m Use `tasks/` manifest to define your current file scope.");
}
