//! A library for serializing and deserializing [Cap'n Proto](https://capnproto.org/) messages with Serde.
//! It relies on the [capnp](https://github.com/capnproto/capnproto-rust) crate.
//! Note that it does *not* use the Cap'n Proto message encoding, this crate is for serializing using *another* format.
//!
//! ## Use Case
//!
//! This crate allows directly converting Cap'n Proto messages to and from another serde-implementing codec,
//! like JSON, YAML or CBOR, without any intermediary data storage.
//!
//! The reason this crate was created is to facilitate two-way translation of Cap'n Proto messages for languages
//! that don't have Cap'n Proto support themselves.
//!
//! Another use case might be easily debugging complex data structures by converting them to JSON to investigate
//! in a JSON viewer. It's also quite easy to quickly construct a Cap'n Proto message via `serde_json::json!`.
//!
//! Note that the Cap'n Proto schema is still required. This crate uses capnp and capnpc like normal, relying
//! on its introspection feature and `capnp::dynamic_value`.
//!
//! ## Limitations
//!
//! Since Cap’n Proto uses arena-style memory allocation and builds the message in-place, it fundamentally requires you to know the size of lists ahead of time. There’s no real way around this with the official Rust capnp crate.
//!
//! Many deserializers supply the array/list size upfront as a hint to the decoder, which solves the problem. However, other decoders do not. There's a workaround implemented for everything except lists and structs, which involves an extra copy of the whole list (via Rust's `Vec<T>`).
//!
//! ## Examples
//!
//! There are a few examples included in the project to demonstrate usage as found in the examples directory.
//! Run them using:
//!
//! ```sh
//! cargo run --example <name> --features examples
//! ```
//!
//! ## License
//!
//! Licensed under either of Apache License, Version 2.0 or MIT license at your option.

mod deserialize;
mod serialize;
mod types;

pub use deserialize::CapnpSerdeBuilder;
pub use serialize::CapnpSerdeReader;
