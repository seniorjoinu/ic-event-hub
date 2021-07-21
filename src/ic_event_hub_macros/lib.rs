#![feature(log_syntax)]

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

#[proc_macro_derive(Event, attributes(topic))]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    event_macro_impl(input)
}

#[proc_macro]
pub fn implement_event_emitter(ts: TokenStream) -> TokenStream {
    implement_event_emitter_impl(ts)
}

#[proc_macro]
pub fn implement_add_event_listeners(ts: TokenStream) -> TokenStream {
    implement_add_event_listeners_impl(ts)
}

#[proc_macro]
pub fn implement_remove_event_listeners(ts: TokenStream) -> TokenStream {
    implement_remove_event_listeners_impl(ts)
}

#[proc_macro]
pub fn implement_become_event_listener(ts: TokenStream) -> TokenStream {
    implement_become_event_listener_impl(ts)
}

#[proc_macro]
pub fn implement_stop_being_event_listener(ts: TokenStream) -> TokenStream {
    implement_stop_being_event_listener_impl(ts)
}

#[proc_macro]
pub fn implement_get_event_listeners(ts: TokenStream) -> TokenStream {
    implement_get_event_listeners_impl(ts)
}
