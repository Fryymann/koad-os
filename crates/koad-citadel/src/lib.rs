//! KoadOS Citadel Core Library
//!
//! The Citadel is the central nervous system of KoadOS, providing gRPC-driven
//! session management, personal bay orchestration, and secure signal routing.
//!
//! This library contains the core kernel, service implementations, and state
//! management logic for the Citadel persistent service.

#![warn(missing_docs)]
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
