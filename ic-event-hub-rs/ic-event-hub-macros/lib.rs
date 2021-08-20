//! Various macro for IC canisters to enable event-based pub/sub
//!
//! Usage:
//! ```
//! // somewhere in your canister
//! implement_event_emitter!();
//! implement_become_event_listener!();
//! implement_stop_being_event_listener!();
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

use proc_macro::TokenStream;

use crate::derive::event_macro_impl;
use crate::implement::{
    implement_add_event_listeners_impl, implement_become_event_listener_impl,
    implement_event_emitter_impl, implement_get_event_listeners_impl,
    implement_remove_event_listeners_impl, implement_stop_being_event_listener_impl,
};

mod derive;
mod implement;
mod parser;

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

/// Generates main functionality for event emitting
///
/// * `get_event_hub()` - returns inner state of event-hub
///   * Returns:
///     * an object of type `ic_event_hub::event_hub::EventHub`, containing all the info about listeners and topics
///
///
/// * `emit()` - sends an event message to all subscribed listeners
///   * Params:
///     * `event`: `ic_event_hub::types::IEvent`
#[proc_macro]
pub fn implement_event_emitter(ts: TokenStream) -> TokenStream {
    implement_event_emitter_impl(ts)
}

/// Generates the canister endpoint to manually subscribe new event listeners
/// ```
/// #[ic_cdk_macros::update]
/// fn _add_event_listeners(request: ic_event_hub::types::AddEventListenersRequest) {}
/// ```
///
/// Params:
/// * `guard`: `str` - optional parameter specifying a guarding function which could perform any access-control functionality
#[proc_macro]
pub fn implement_add_event_listeners(ts: TokenStream) -> TokenStream {
    implement_add_event_listeners_impl(ts)
}

/// Generates the canister endpoint to manually unsubscribe previously subscribed listeners
/// ```
/// #[ic_cdk_macros::update]
/// fn _remove_event_listeners(request: ic_event_hub::types::RemoveEventListenersRequest) {}
/// ```
///
/// Params:
/// * `guard`: `str` - optional parameter specifying a guarding function which could perform any access-control functionality
#[proc_macro]
pub fn implement_remove_event_listeners(ts: TokenStream) -> TokenStream {
    implement_remove_event_listeners_impl(ts)
}

/// Generates the canister endpoint for another canister to subscribe to particular topics
/// ```
/// #[ic_cdk_macros::update]
/// fn _become_event_listener(request: ic_event_hub::types::BecomeEventListenerRequest) {}
/// ```
///
/// Params:
/// * `guard`: `str` - optional parameter specifying a guarding function which could perform any access-control functionality
#[proc_macro]
pub fn implement_become_event_listener(ts: TokenStream) -> TokenStream {
    implement_become_event_listener_impl(ts)
}

/// Generates the canister endpoint for another canister to unsubscribe from particular topics
/// ```
/// #[ic_cdk_macros::update]
/// fn _stop_being_event_listener(request: ic_event_hub::types::StopBeingEventListenerRequest) {}
/// ```
///
/// Params:
/// * `guard`: `str` - optional parameter specifying a guarding function which could perform any access-control functionality
#[proc_macro]
pub fn implement_stop_being_event_listener(ts: TokenStream) -> TokenStream {
    implement_stop_being_event_listener_impl(ts)
}

/// Generates the canister endpoint to list all the subscribed listeners
/// ```
/// #[ic_cdk_macros::query]
/// fn _get_event_listeners(
///     request: ic_event_hub::types::GetEventListenersRequest
/// ) -> ic_event_hub::types::GetEventListenersResponse {}
/// ```
///
/// Params:
/// * `guard`: `str` - optional parameter specifying a guarding function which could perform any access-control functionality
#[proc_macro]
pub fn implement_get_event_listeners(ts: TokenStream) -> TokenStream {
    implement_get_event_listeners_impl(ts)
}
