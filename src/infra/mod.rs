//! Infrastructure implementations.
//!
//! This module contains concrete implementations of the infrastructure
//! traits defined in `core::infra`.
//!
//! # Implementations
//!
//! - [`StdFileSystem`] - File system using standard library
//! - [`ReqwestClient`] - HTTP client using reqwest
//! - [`Sha2Checksum`] - SHA256 checksum using sha2
//! - [`ColoredLogger`] - Colored terminal output

mod checksum;
mod fs;
mod http;
mod logger;

pub use checksum::Sha2Checksum;
pub use fs::StdFileSystem;
pub use http::ReqwestClient;
pub use logger::ColoredLogger;
