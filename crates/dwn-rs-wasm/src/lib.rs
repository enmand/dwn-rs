#![cfg_attr(feature = "no-std", no_std)]

#[cfg(feature = "no-std")]
extern crate alloc;
#[cfg(not(feature = "no-std"))]
extern crate std as alloc;

mod data;
mod filter;
mod message;
mod query;
pub(crate) mod ser;
mod streams;
mod task;
mod tracing;

#[cfg(feature = "surrealdb")]
mod surrealdb;
