<aside>
📖

**Yellow Pages for KoadOS Agents.** When you need authoritative information on a topic, look it up here first. Find the category, pick the preferred source, then fetch what you need. Do not guess or hallucinate docs — use these links.

</aside>

> **How to use:** Find your topic's category → check Preferred Source(s) first → fall back to Secondary if needed → note any KoadOS-specific guidance in the Notes column.
> 

---

## 🦀 Rust

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| Rust language reference | [The Rust Reference](https://doc.rust-lang.org/reference/) | [The Rust Book](https://doc.rust-lang.org/book/) | Use the Reference for precise language semantics; use the Book for conceptual learning. |
| Standard library (std) | [std docs —](https://doc.rust-lang.org/std/) [doc.rust-lang.org/std](http://doc.rust-lang.org/std) | [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Search a type or trait directly: `doc.rust-lang.org/std/primitive.<type>.html` |
| Async / Tokio | [Tokio docs](https://docs.rs/tokio/latest/tokio/) | [Async Rust book](https://rust-lang.github.io/async-book/) | KoadOS uses async heavily in the Citadel and agent pipelines. |
| Error handling | [thiserror docs](https://docs.rs/thiserror/latest/thiserror/) | [anyhow docs](https://docs.rs/anyhow/latest/anyhow/) | Prefer `thiserror` for library errors, `anyhow` for application-level propagation. |
| Lifetimes & borrowing | [The Rust Book — Ch. 10](https://doc.rust-lang.org/book/ch10-00-generics.html) | [Rustonomicon](https://doc.rust-lang.org/nomicon/) | Nomicon for unsafe edge cases only. |
| Macros | [The Rust Reference — Macros](https://doc.rust-lang.org/reference/macros.html) | [The Little Book of Rust Macros](https://veykril.github.io/tlborm/) |  |
| Traits & generics | [The Rust Book — Ch. 10](https://doc.rust-lang.org/book/ch10-00-generics.html) | [std trait index](https://doc.rust-lang.org/std/#traits) |  |

---

## 📦 Cargo & Tooling

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| Cargo general | [The Cargo Book](https://doc.rust-lang.org/cargo/) | [cargo CLI reference](https://doc.rust-lang.org/cargo/commands/) | Authoritative source for manifest, workspace, and dependency config. |
| Code coverage — `cargo-llvm-cov` | [cargo-llvm-cov README](https://github.com/taiki-e/cargo-llvm-cov) | [docs.rs/cargo-llvm-cov](http://docs.rs/cargo-llvm-cov) | KoadOS canonical coverage tool. Use `--lcov` output for CI. |
| TUI — `ratatui` | [ratatui docs](https://docs.rs/ratatui/latest/ratatui/) | [ratatui book](https://ratatui.rs/) | KoadOS canonical TUI library. Start with the book for layout concepts. |
| Docs generation — `cargo doc` | [rustdoc reference](https://doc.rust-lang.org/rustdoc/) | [Cargo Book — cargo doc](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) | Run `cargo doc --open` locally. Use `cargo-deadlinks` to validate. |
| Dead link checking — `cargo-deadlinks` | [cargo-deadlinks](https://crates.io/crates/cargo-deadlinks) [crates.io](http://crates.io) | [cargo-deadlinks GitHub](https://github.com/deadlinks/cargo-deadlinks) | Run after `cargo doc` to validate all intra-doc links. |
| Any crate lookup | [docs.rs](http://docs.rs) | [crates.io](http://crates.io) | Always use `docs.rs/<crate>/<version>/<crate>/` for the canonical API reference. |

---

## 🗄️ Data & Storage

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| SQLite | [SQLite official docs](https://www.sqlite.org/docs.html) | [rusqlite docs](https://docs.rs/rusqlite/latest/rusqlite/) | KoadOS memory layer. Use rusqlite for Rust bindings. |
| Redis | [Redis commands reference](https://redis.io/commands/) | [redis-rs docs](https://docs.rs/redis/latest/redis/) | KoadOS hot memory / pub-sub. Use redis-rs for Rust. |
| Qdrant (vector DB) | [Qdrant docs](https://qdrant.tech/documentation/) | [qdrant-client Rust docs](https://docs.rs/qdrant-client/latest/qdrant_client/) | KoadOS semantic memory layer. |

---

## ☁️ Cloud & Infrastructure

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| Google Cloud Functions (GCF) | [GCF docs](https://cloud.google.com/functions/docs) | [GCF Node.js samples](https://github.com/GoogleCloudPlatform/nodejs-docs-samples/tree/main/functions) | SWS primary compute layer. |
| Google Cloud general | [GCP product docs](https://cloud.google.com/docs) | [GCP Node.js client libraries](https://cloud.google.com/nodejs/docs/reference) |  |
| Stripe | [Stripe API docs](https://stripe.com/docs/api) | [Stripe webhooks guide](https://stripe.com/docs/webhooks) | SWS payment pipeline. Webhook → GCF → Airtable flow. |
| Airtable | [Airtable Web API reference](https://airtable.com/developers/web/api/introduction) | [Airtable field types](https://airtable.com/developers/web/api/field-model) | SWS operational database. Use the auto-generated base-specific docs at `airtable.com/appXXX/api/docs`. |
|  |  |  |  |

---

## 🤖 AI / LLM APIs

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| Gemini API | [Gemini API docs](https://ai.google.dev/gemini-api/docs) | [Gemini model reference](https://ai.google.dev/gemini-api/docs/models/gemini) | KoadOS primary LLM API. See also the Gemini API Models database in the KoadOS hub. |
| OpenAI API | [OpenAI API reference](https://platform.openai.com/docs/api-reference) | [OpenAI guides](https://platform.openai.com/docs/guides) | Secondary / fallback LLM provider. |
| Prompt engineering | [Google prompt engineering guide](https://ai.google.dev/gemini-api/docs/prompting-strategies) | [OpenAI prompt engineering guide](https://platform.openai.com/docs/guides/prompt-engineering) | See also Ian's internal Promptwork page for KoadOS-specific patterns. |

---

## 🌐 Node.js / JavaScript

| **Topic** | **Preferred Source** | **Secondary / Supplemental** | **Notes** |
| --- | --- | --- | --- |
| Node.js built-in APIs | [Node.js API docs](https://nodejs.org/api/) | [Node.js guides](https://nodejs.org/en/learn) | Pin to the active LTS version in use. |
| npm packages | [npmjs.com](http://npmjs.com) | Package's own GitHub README | Always check the package's own docs link from its npm page. |
| MDN (browser / web standard APIs) | [MDN Web Docs](https://developer.mozilla.org/) |  | Authoritative for Web APIs, Fetch, WebSocket, etc. |

---

## 🔧 KoadOS-Internal References

<aside>
🔒

These are internal Notion pages — agents must have Notion access to read them.

</aside>

| **Topic** | **Source** | **Notes** |
| --- | --- | --- |
| Global Canon & Rules of Engagement | [**KoadOS Global Canon & Rules of Engagement**](https://www.notion.so/KoadOS-Global-Canon-Rules-of-Engagement-319fe8ecae8f8064a430c2a677a62188?pvs=21) | Prime authority for all KoadOS agent behavior rules. |
| Rust best practices (KoadOS-specific) | [Rust Best Practices & Cargo Guide — KoadOS Development Canon](https://www.notion.so/Rust-Best-Practices-Cargo-Guide-KoadOS-Development-Canon-c7a2ebc9049f4e089744f12f24c2425a?pvs=21) | Check here before external Rust docs for KoadOS-opinionated guidance. |
| Contributor & coding manifesto | [KoadOS — Contributor & Coding Manifesto](https://www.notion.so/KoadOS-Contributor-Coding-Manifesto-318fe8ecae8f81c3afd3e82a7fee0b8a?pvs=21) | Code style, commit conventions, PR process. |
| Agent prompt library | [KoadOS — Agent Prompt Library](https://www.notion.so/KoadOS-Agent-Prompt-Library-73f94d8e2f3545d5a496a21ad45594ee?pvs=21) | Canonical prompts for agent initialization, bootstrap, and task delegation. |
| Koad Stream protocol | [Koad Stream — Protocol](https://www.notion.so/Koad-Stream-Protocol-4ba0dafed6bb4526bda44680993b450f?pvs=21) | Message bus spec for Koad ↔ Noti and inter-agent comms. |
| Citadel refactor research | [Citadel Refactor — Brainstorm & Research](https://www.notion.so/Citadel-Refactor-Brainstorm-Research-ff598ede2a0048998e3262119fd13cef?pvs=21) | Active architectural context for Citadel work. |

---

## 🎖️ Agent Levels & Trust

XP defines an agent's trust tier and eligibility for complex operations.

| Level | Title | XP Threshold | Trust Descriptor |
| :--- | :--- | :--- | :--- |
| 1 | Initiate | 0 XP | Base Canon access; full gate oversight |
| 2 | Operative | 50 XP | Trusted for `trivial` tasks with lighter check-in cadence |
| 3 | Sentinel | 150 XP | Eligible to propose Canon amendments in `ponder` passes |
| 4 | Architect | 350 XP | Eligible for `complex` task leads |
| 5 | Sovereign | 600 XP | Full trust tier; advisory role on Canon evolution |

> **Note:** Levels are descriptive, not prescriptive. They never bypass Canon gates or override Admiral (Ian) authority.

---

## 🚀 KoadOS Boot & Hydration

| **Topic** | **Source** | **Notes** |
| --- | --- | --- |
| Agent Boot Process | `koad-agent boot <AgentName>` | Generates required environment variables and context for the active agent session. |
| Dynamic Context Anchor | `~/.gemini/GEMINI.md` / `~/.claude/CLAUDE.md` | Dynamically overwritten at boot. Do NOT rely on this file as a persistent source of truth. |
| Session Brief | `~/.koad-os/cache/session-brief-<agent>.md` | Auto-generated at boot. Contains Git status, recent commits, and your `WORKING_MEMORY.md`. **Read this first after booting.** |
| Identity Definitions | `~/.koad-os/config/identities/*.toml` | The canonical source of truth for agent roles, ranks, and preferences. |

---

## 📋 Lookup Protocol (Agent Usage)

1. **Identify topic category** from the sections above.
2. **Check KoadOS-Internal first** if the topic is KoadOS-specific behavior or conventions.
3. **Use Preferred Source** for the topic. Load the URL and retrieve the relevant section.
4. **Fall back to Secondary** only if the primary source doesn't cover the specific question.
5. **If no entry exists** for the topic: flag it for addition to this reference book and proceed with a web search as a temporary measure.
6. *(Future)* Delegate to a support/lookup micro-agent which will hydrate the result directly into the calling agent's context.

---

<aside>
✏️

**Maintainers: Noti + Scribe.** Add new topics as they arise during KoadOS development. Keep Preferred Source authoritative — don't add a source unless it's consistently reliable for KoadOS work.

</aside>