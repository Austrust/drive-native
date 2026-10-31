#![doc = include_str!("../README.md")]

// pstream must come first so the pmap! macro is available to other modules.
#[macro_use]
pub(crate) mod pstream;

pub mod actions;
pub(crate) mod channel;
pub mod client;
pub(crate) mod error;
pub(crate) mod frame;
pub(crate) mod pool;

pub use channel::TlsMode;
pub use client::Client;
pub use error::{Error, Result};

#[cfg(feature = "protocol-research")]
pub mod research;
#[cfg(feature = "protocol-research")]
pub use pstream::PObject;

pub mod trust;

#[cfg(feature = "protocol-research")]
pub mod trust_relay;

pub mod flow;
pub mod receive_activity;
