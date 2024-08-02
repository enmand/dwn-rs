//! Decentralized Web Node core library.
//!
//! This library provides core traits and implementations to interact with Decentralized
//! Web Nodes (DWN) services. It is intended to be used by other libraries and applications
//! that need to interact with DWN services.
//!
//! The library is designed to be used in a variety of environments, including WebAssembly
//! and native applications. It is written in Rust and can be compiled to WebAssembly for
//! use in web browsers. See the `dwn-rs-wasm` crate for more information on using this.
//!
//! The primary features of this library are the core DWN types and traits, including:
//! - [`messages::<D: Descriptor, F: Fields>`]
//! - [`messages::Descriptor`]: A descriptor for a message.
//! - [`messages::Fields`]: Additional fields that can be included in a message.
//! - [`value::Value`]: A generic value type that can be used in messages.
//! - [`value::MapValue`]: A map of values that can be used in messages.`
//!
//! Additionally, there are strongly typed values for common DWN messages, including:
//! - [`messages::records::RecordsRead`]: A message for reading records.
//! - [`messages::records::RecordsQuery`]: A descriptor for reading records.
//! - [`messages::records::RecordsWrite`]: A descriptor for reading records.
//! - [`messages::records::RecordsSubscribe`]: A descriptor for reading records.
//! - [`messages::records::RecordsDelete`]: A descriptor for reading records.
#![doc(issue_tracker_base_url = "https://github.com/enmand/dwn-rsissues/")]
pub mod auth;
pub mod errors;
pub mod filters;
pub mod interfaces;
mod serde;
pub mod stores;
pub mod value;

pub use filters::*;
pub use interfaces::*;
pub use value::*;
