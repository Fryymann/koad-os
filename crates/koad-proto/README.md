# koad-proto

The communication protocols for the **KoadOS** ecosystem. This crate manages gRPC and Tonic-generated service definitions used by the Citadel, CLI, and Bridge agents.

## 🏗 Overview

`koad-proto` acts as the central contract for component interoperability. It contains the Protocol Buffer definitions and their generated Rust bindings.

## 🛰 Core Services

- **`citadel.v5`**: The primary Citadel gRPC interface for session management, context hydration, and heartbeat monitoring.
- **`skill`**: Service definitions for specialized agent skill execution and communication.

## 🛠 Protos

The raw `.proto` definitions are located in the root `proto/` directory:
- `proto/citadel.proto`: State orchestration, session monitoring, and context management.
- `proto/skill.proto`: Agent-to-Skill communication protocols.

## ⚙️ Build Process

The crate utilizes `tonic-build` in its `build.rs` to compile the proto definitions during the build cycle. Ensure `protoc` is available in your environment.

```rust
// build.rs
tonic_build::configure().compile_protos(
    &["../../proto/skill.proto", "../../proto/citadel.proto"],
    &["../../proto"],
)?;
```

## 🚀 Usage

```toml
[dependencies]
koad-proto = { path = "../koad-proto" }
```
