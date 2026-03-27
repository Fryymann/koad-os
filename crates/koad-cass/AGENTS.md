# Crate: koad-cass
**Status:** Complete (Agent Support System)
**Port:** :50052

## Purpose
The Citadel Agent Support System (CASS) provides cognitive infrastructure for agents. It handles memory persistence, context hydration, End-of-Watch reporting, and hosts the Model Context Protocol (MCP) tool registry.

## Source Map
- `main.rs`: Entry point; initializes the CASS server.
- `services/`:
    - `hydration.rs`: Temporal Context Hydrator (TCH); builds the distilled `current_context.md`.
    - `memory.rs`: Knowledge retrieval and FactCard management.
    - `eow.rs`: End-of-Watch pipeline; automated session summarization.
    - `tool_registry.rs`: MCP Tool Registry; hosts WASM-based agent tools.
    - `symbol.rs`: Integration with the CodeGraph symbol index.
- `storage/`: Backend persistence for CASS cognitive state.

## Public API (gRPC)
- `HydrationServer`: TCH context delivery.
- `MemoryServer`: Fact and knowledge base queries.
- `ToolRegistryServer`: Dynamic tool registration and invocation.
- `EowServer`: Session closure and reporting.

## Dependencies
- `koad-core`: Shared types and config.
- `koad-proto`: gRPC service definitions.
- `koad-plugins`: WASM runtime for MCP tools.
- `koad-intelligence`: Model routing and distillation.
