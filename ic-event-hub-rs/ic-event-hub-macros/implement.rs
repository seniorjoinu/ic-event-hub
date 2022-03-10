use proc_macro::TokenStream;

use quote::quote;
use syn::parse;

use crate::parser::GuardAssign;

pub fn implement_event_emitter_impl(_: TokenStream) -> TokenStream {
    let gen = quote! {
        static mut _EVENT_HUB: Option<ic_event_hub::event_hub::EventHub> = None;

        pub fn get_event_hub() -> &'static mut ic_event_hub::event_hub::EventHub {
            unsafe {
                if let Some(s) = &mut _EVENT_HUB {
                    s
                } else {
                    _EVENT_HUB = Some(ic_event_hub::event_hub::EventHub::new());
                    get_event_hub()
                }
            }
        }

        pub fn emit(event: impl ic_event_hub::types::IEvent) {
            ic_event_hub::fns::emit_impl(event, get_event_hub());
        }

        pub fn send_events() {
            ic_event_hub::fns::send_events_impl(get_event_hub());
        }
    };

    gen.into()
}

pub fn implement_add_event_listeners_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _add_event_listeners(request: ic_event_hub::types::AddEventListenersRequest) {
            let hub = get_event_hub();

            ic_event_hub::fns::add_event_listeners_impl(request, hub);
        }
    };

    gen.into()
}

pub fn implement_remove_event_listeners_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _remove_event_listeners(request: ic_event_hub::types::RemoveEventListenersRequest) {
            let hub = get_event_hub();

            ic_event_hub::fns::remove_event_listeners_impl(request, hub);
        }
    };

    gen.into()
}

pub fn implement_become_event_listener_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _become_event_listener(request: ic_event_hub::types::BecomeEventListenerRequest) {
            let hub = get_event_hub();

            ic_event_hub::fns::become_event_listener_impl(request, hub);
        }
    };

    gen.into()
}

pub fn implement_stop_being_event_listener_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _stop_being_event_listener(request: ic_event_hub::types::StopBeingEventListenerRequest) {
            let hub = get_event_hub();

            ic_event_hub::fns::stop_being_event_listener_impl(request, hub);
        }
    };

    gen.into()
}

pub fn implement_get_event_listeners_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_query_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _get_event_listeners(
            request: ic_event_hub::types::GetEventListenersRequest,
        ) -> ic_event_hub::types::GetEventListenersResponse {
            let hub = get_event_hub();

            ic_event_hub::fns::get_event_listeners_impl(request, hub)
        }
    };

    gen.into()
}

fn generate_ic_update_macro(ts: TokenStream) -> proc_macro2::TokenStream {
    let expr = parse::<GuardAssign>(ts).unwrap();

    match expr.guard_name {
        Some(name) => quote! {
            #[ic_cdk_macros::update(guard = #name)]
        },
        None => quote! {
            #[ic_cdk_macros::update]
        },
    }
}

fn generate_ic_query_macro(ts: TokenStream) -> proc_macro2::TokenStream {
    let expr = parse::<GuardAssign>(ts).unwrap();

    match expr.guard_name {
        Some(name) => quote! {
            #[ic_cdk_macros::query(guard = #name)]
        },
        None => quote! {
            #[ic_cdk_macros::query]
        },
    }
}
