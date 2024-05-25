//! A Rust library for interacting with the Bluesim simulation program.
#![warn(clippy::unwrap_used)]

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test;

mod config;
mod server;

pub use config::*;
pub use server::*;
