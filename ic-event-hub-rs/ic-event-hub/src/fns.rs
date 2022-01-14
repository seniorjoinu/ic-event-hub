use crate::event_hub::EventHub;
use crate::types::{
    AddEventListenersRequest, BecomeEventListenerRequest, GetEventListenersRequest,
    GetEventListenersResponse, IEvent, RemoveEventListenersRequest, StopBeingEventListenerRequest,
};
use futures::future;
use ic_cdk::api::call::call_raw;
use ic_cdk::export::candid::encode_args;
use ic_cdk::{block_on, caller, id, print, trap};
use union_utils::log;

pub fn emit_impl(event: impl IEvent, hub: &mut EventHub) {
    print(format!("[Canister {}] - ic_event_hub.emit()", id()));

    print(format!("emit - {:?}", hub.listeners));

    hub.push_pending_event(event.to_event());
}

pub fn send_events_impl(batch_size: usize, hub: &mut EventHub) {
    let mut emit_futures = vec![];

    loop {
        let events_opt = hub.pop_pending_events();

        if events_opt.is_none() {
            break;
        }

        print(format!(
            "[Canister {}]: heartbeat - ic_event_hub.send_events()",
            id()
        ));

        let (endpoint, events) = events_opt.unwrap();

        for event_batch in events.chunks(batch_size) {
            let payload = encode_args((event_batch,)).unwrap();

            let future = call_raw(
                endpoint.canister_id,
                endpoint.method_name.as_str(),
                payload,
                0,
            );

            emit_futures.push(future);
        }
    }

    if !emit_futures.is_empty() {
        block_on(async {
            future::join_all(emit_futures).await;
        });
    }
}

pub fn add_event_listeners_impl(request: AddEventListenersRequest, hub: &mut EventHub) {
    log("ic_event_hub._add_event_listeners()");

    for listener in request.listeners.into_iter() {
        hub.add_event_listener(
            listener.filter,
            listener.endpoint.method_name,
            listener.endpoint.canister_id,
        );
    }
}

pub fn remove_event_listeners_impl(request: RemoveEventListenersRequest, hub: &mut EventHub) {
    log("ic_event_hub._remove_event_listeners()");

    for (idx, listener) in request.listeners.into_iter().enumerate() {
        let res = hub.remove_event_listener(
            &listener.filter,
            listener.endpoint.method_name,
            listener.endpoint.canister_id,
        );

        if res.is_err() {
            trap(
                format!(
                    "Unable to remove listener #{} - {}",
                    idx,
                    res.err().unwrap()
                )
                .as_str(),
            );
        }
    }
}

pub fn become_event_listener_impl(request: BecomeEventListenerRequest, hub: &mut EventHub) {
    log("ic_event_hub._become_event_listener()");

    for listener in request.listeners.into_iter() {
        hub.add_event_listener(listener.filter, listener.callback_method_name, caller());
    }

    print(format!("become event listener {:?}", hub.listeners));
}

pub fn stop_being_event_listener_impl(request: StopBeingEventListenerRequest, hub: &mut EventHub) {
    log("ic_event_hub._stop_being_event_listener()");

    for (idx, listener) in request.listeners.into_iter().enumerate() {
        let res =
            hub.remove_event_listener(&listener.filter, listener.callback_method_name, caller());

        if res.is_err() {
            trap(
                format!(
                    "Unable to remove listener #{} - {}",
                    idx,
                    res.err().unwrap()
                )
                .as_str(),
            );
        }
    }
}

pub fn get_event_listeners_impl(
    request: GetEventListenersRequest,
    hub: &mut EventHub,
) -> GetEventListenersResponse {
    log("ic_event_hub._get_event_listeners()");

    let mut listeners = vec![];

    for filter in request.filters.iter() {
        listeners.push(hub.match_event_listeners(filter));
    }

    GetEventListenersResponse { listeners }
}
