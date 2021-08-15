use proc_macro::TokenStream;

use quote::quote;
use syn::parse;

use crate::parser::GuardAssign;

pub fn implement_event_emitter_impl(_: TokenStream) -> TokenStream {
    let gen = quote! {
        static mut _EVENT_HUB: Option<ic_event_hub::event_hub::EventHub> = None;

        #[inline(always)]
        pub fn get_event_hub() -> &'static mut ic_event_hub::event_hub::EventHub {
            unsafe {
                if let Some(s) = &mut _EVENT_HUB {
                    s
                } else {
                    _EVENT_HUB = Some(ic_event_hub::event_hub::EventHub::default());
                    get_event_hub()
                }
            }
        }

        pub fn emit(event: impl ic_event_hub::types::IEvent) {
            union_utils::log("ic_event_hub.emit()");

            let ev = event.to_event();
            let hub = get_event_hub();

            let listeners = hub.match_event_listeners_by_topics(&ev.topics);
            if listeners.is_empty() {
                return;
            }

            let event_raw = ic_cdk::export::candid::encode_args((ev.clone(),)).unwrap();
            let mut emit_futures = vec![];

            for listener in listeners.iter() {
                let future = ic_cdk::api::call::call_raw(
                    listener.canister_id,
                    listener.method_name.as_str(),
                    event_raw.clone(),
                    0,
                );

                emit_futures.push(future);
            }

            if !emit_futures.is_empty() {
                ic_cdk::block_on(async {
                    futures::future::join_all(emit_futures).await;
                });
            }
        }
    };

    gen.into()
}

pub fn implement_add_event_listeners_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _add_event_listeners(request: ic_event_hub::types::AddEventListenersRequest) {
            union_utils::log("ic_event_hub._add_event_listeners()");

            let hub = get_event_hub();

            for listener in request.listeners.into_iter() {
                hub.add_event_listener(
                    listener.filter,
                    listener.endpoint.method_name,
                    listener.endpoint.canister_id,
                );
            }
        }
    };

    gen.into()
}

pub fn implement_remove_event_listeners_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _remove_event_listeners(
            request: ic_event_hub::types::RemoveEventListenersRequest,
        ) -> ic_event_hub::types::RemoveEventListenersResponse {
            union_utils::log("ic_event_hub._remove_event_listeners()");

            let hub = get_event_hub();
            let mut results = vec![];

            for listener in request.listeners.into_iter() {
                results.push(hub.remove_event_listener(
                    &listener.filter,
                    listener.endpoint.method_name,
                    listener.endpoint.canister_id,
                ));
            }

            ic_event_hub::types::RemoveEventListenersResponse { results }
        }
    };

    gen.into()
}

pub fn implement_become_event_listener_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _become_event_listener(request: ic_event_hub::types::BecomeEventListenerRequest) {
            union_utils::log("ic_event_hub._become_event_listener()");

            let hub = get_event_hub();

            for listener in request.listeners.into_iter() {
                hub.add_event_listener(
                    listener.filter,
                    listener.callback_method_name,
                    ic_cdk::caller(),
                );
            }
        }
    };

    gen.into()
}

pub fn implement_stop_being_event_listener_impl(ts: TokenStream) -> TokenStream {
    let ic_macro = generate_ic_update_macro(ts);

    let gen = quote! {
        #ic_macro
        fn _stop_being_event_listener(
            request: ic_event_hub::types::StopBeingEventListenerRequest,
        ) -> ic_event_hub::types::StopBeingEventListenerResponse {
            union_utils::log("ic_event_hub._stop_being_event_listener()");

            let hub = get_event_hub();
            let mut results = vec![];

            for listener in request.listeners.into_iter() {
                results.push(hub.remove_event_listener(
                    &listener.filter,
                    listener.callback_method_name,
                    ic_cdk::caller(),
                ));
            }

            ic_event_hub::types::StopBeingEventListenerResponse { results }
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
            union_utils::log("ic_event_hub._get_event_listeners()");

            let hub = get_event_hub();
            let mut listeners = vec![];

            for filter in request.filters.iter() {
                listeners.push(hub.match_event_listeners(filter));
            }

            ic_event_hub::types::GetEventListenersResponse { listeners }
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
