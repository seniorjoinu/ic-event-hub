use crate::event_hub::EventHub;
use crate::types::{
    AddEventListenersRequest, BecomeEventListenerRequest, Event, GetEventListenersRequest,
    GetEventListenersResponse, IEvent, RemoveEventListenersRequest, StopBeingEventListenerRequest,
};
use candid::ser::TypeSerialize;
use candid::CandidType;
use futures::future;
use ic_cdk::api::call::call_raw;
use ic_cdk::{caller, id, print, trap};

pub fn emit_impl(event: impl IEvent, hub: &mut EventHub) {
    print(format!("[Canister {}] - ic_event_hub.emit()", id()));

    hub.push_pending_event(event.to_event());
}

pub fn send_events_impl(hub: &mut EventHub) {
    let mut emit_futures = vec![];

    loop {
        let batches_opt = hub.pop_pending_events();

        if batches_opt.is_none() {
            break;
        }

        let (endpoint, batches) = batches_opt.unwrap();

        print(format!(
            "[Canister {}]: heartbeat - ic_event_hub.send_events()",
            id()
        ));

        let mut type_ser = TypeSerialize::new();
        type_ser
            .push_type(&Vec::<Event>::ty())
            .expect("Unable to push type");
        type_ser.serialize().expect("Unable to serialize types");

        for batch in batches {
            let mut msg: Vec<u8> = vec![];
            msg.extend_from_slice(b"DIDL");
            msg.extend_from_slice(type_ser.get_result());
            leb128::write::unsigned(&mut msg, batch.events_count as u64)
                .expect("Unable to write len");
            msg.extend_from_slice(&batch.content);

            emit_futures.push(call_raw(
                endpoint.canister_id,
                endpoint.method_name.as_str(),
                msg,
                0,
            ));
        }
    }

    if !emit_futures.is_empty() {
        ic_cdk::block_on(async {
            future::join_all(emit_futures).await;
        });
    }
}

pub fn add_event_listeners_impl(request: AddEventListenersRequest, hub: &mut EventHub) {
    for listener in request.listeners.into_iter() {
        hub.add_event_listener(
            listener.filter,
            listener.endpoint.method_name,
            listener.endpoint.canister_id,
        );
    }
}

pub fn remove_event_listeners_impl(request: RemoveEventListenersRequest, hub: &mut EventHub) {
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
    for listener in request.listeners.into_iter() {
        hub.add_event_listener(listener.filter, listener.callback_method_name, caller());
    }
}

pub fn get_event_listeners_impl(
    request: GetEventListenersRequest,
    hub: &mut EventHub,
) -> GetEventListenersResponse {
    let mut listeners = vec![];

    for filter in request.filters.iter() {
        listeners.push(hub.match_event_listeners(filter));
    }

    GetEventListenersResponse { listeners }
}

pub fn stop_being_event_listener_impl(request: StopBeingEventListenerRequest, hub: &mut EventHub) {
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

#[cfg(test)]
mod tests {
    use candid::ser::{TypeSerialize, ValueSerializer};
    use candid::{encode_args, encode_one, CandidType, Nat};

    #[test]
    fn tst() {
        let v1 = Nat::from(3212312312u64);
        let v2 = String::from("Kek");

        let kek1 = encode_args((v1.clone(), v2.clone())).expect("Unable to encode args");

        let mut type_ser = TypeSerialize::new();
        type_ser.push_type(&Nat::ty()).unwrap();
        type_ser.push_type(&String::ty()).unwrap();
        type_ser.serialize().unwrap();

        let mut value_ser = ValueSerializer::new();
        v1.idl_serialize(&mut value_ser).unwrap();
        v2.idl_serialize(&mut value_ser).unwrap();

        let mut kek2 = vec![];
        kek2.extend_from_slice(b"DIDL");
        kek2.extend_from_slice(type_ser.get_result());
        kek2.extend_from_slice(value_ser.get_result());

        assert_eq!(kek1, kek2, "Keks not equal");

        let v1 = Nat::from(123123123u64);
        let v2 = Nat::from(4312412341u64);
        let v3 = Nat::from(6456464554u64);

        let kek1 =
            encode_one(vec![v1.clone(), v2.clone(), v3.clone()]).expect("Unable to encode args");

        let mut type_ser = TypeSerialize::new();
        type_ser.push_type(&Vec::<Nat>::ty()).unwrap();
        type_ser.serialize().unwrap();

        let mut value_ser = ValueSerializer::new();
        v1.idl_serialize(&mut value_ser).unwrap();
        v2.idl_serialize(&mut value_ser).unwrap();
        v3.idl_serialize(&mut value_ser).unwrap();

        let mut kek2 = vec![];
        kek2.extend_from_slice(b"DIDL");
        kek2.extend_from_slice(type_ser.get_result());
        leb128::write::unsigned(&mut kek2, 3).unwrap();
        kek2.extend_from_slice(value_ser.get_result());

        assert_eq!(kek1, kek2, "Keks not equal 2");
    }
}
