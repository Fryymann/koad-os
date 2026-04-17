//! KoadOS Citadel Core Library
//!
//! The Citadel is the central nervous system of KoadOS, providing gRPC-driven
//! session management, personal bay orchestration, and secure signal routing.
//!
//! This library contains the core kernel, service implementations, and state
//! management logic for the Citadel persistent service.

// missing_docs is suppressed in test mode due to a rustc 1.93.1 ICE (StyledBuffer::replace
// out-of-bounds) triggered by the MissingDoc lint pass on test binaries. The warning
// remains active for all non-test builds (cargo build, cargo check, cargo clippy).
#![cfg_attr(not(test), warn(missing_docs))]
#![forbid(unsafe_code)]

pub mod admin;
pub mod auth;
pub mod kernel;
pub mod services;
pub mod signal_corps;
pub mod state;
pub mod workspace;

/// Re-export the kernel builder for the binary entry point.
pub use kernel::KernelBuilder;
