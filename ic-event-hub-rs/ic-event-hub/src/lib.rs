//! Accompanying library for [ic-event-hub-macros](https://crates.io/crates/ic-event-hub-macros)

#![warn(missing_docs)]

/// Client struct used to interact with canisters which implement `event-emitter` with type-safety
pub mod api;

/// Event-hub struct that handles listeners indexing and topic matching
pub mod event_hub;

/// Various structs and traits
pub mod types;

/// Marker that enables event name serialization
pub const EVENT_NAME_FIELD: &str = "__event_name";
