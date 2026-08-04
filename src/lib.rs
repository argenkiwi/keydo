//! keydo core library — pure-logic modules shared by the daemon binary and the
//! WASM playground crate.
//!
//! This crate root exposes only the platform-agnostic modules so that
//! `keydo-wasm` (in the separate `keydo-playground` repo) can depend on
//! `keydo` as a git dependency and target `wasm32-unknown-unknown` without
//! pulling in any OS-specific code.
//!
//! The daemon binary (`main.rs`) declares the same modules independently as
//! its own crate root; this library crate is a separate compilation unit.

pub mod config;
pub mod config_impl;
pub mod config_parse;
pub mod config_validate;
pub mod error;
pub mod ini;
pub mod keyboard_impl;
pub mod keyboard_types;
pub mod keys;
pub mod macro_parse;
pub mod macro_types;
pub mod unicode;

#[cfg(test)]
pub mod test_io;
#[cfg(test)]
pub mod tests;
