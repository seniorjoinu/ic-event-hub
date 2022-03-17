//! Various macro for IC canisters to enable event-based pub/sub
//!
//! Usage:
//! ```
//! // somewhere in your canister
//! implement_event_emitter!();
//! implement_subscribe!();
//! implement_unsubscribe!();
//!
//! ...
//!
//! #[derive(Event)]
//! struct Event {
//!     #[topic]
//!     pub a: u64,
//!     pub b: String,
//! }
//!
//! emit(Event {
//!     a: 10,
//!     b: String::from("test")
//! });
//! ```
//!
//! Check the [companion crate](https://crates.io/crates/ic-event-hub) to see how a listener could
//! start receiving events

#![warn(missing_docs)]

use crate::derive::event_macro_impl;
use proc_macro::TokenStream;

mod derive;

/// Generates an implementation of `ic_event_hub::types::IEvent` trait for a given struct. Also generates a `*Filter`
/// struct and an implementation of `ic_event_hub::types::IEventFilter` trait for that struct which can be used to filter
/// topics while listening to the given event.
///
/// Fields of the event struct have to implement `candid::CandidType` and `candid::Deserialize`
///
/// Usage:
/// ```
/// #[derive(Event)]
/// struct MyEvent {
///     ...
/// }
/// ```
#[proc_macro_derive(Event, attributes(topic))]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    event_macro_impl(input)
}
