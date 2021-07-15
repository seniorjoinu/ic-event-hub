#[macro_export]
macro_rules! implement_event_emitter {
    () => {
        static mut _EVENT_HUB: Option<ic_event_hub::EventHub> = None;

        pub fn get_event_hub() -> &'static mut ic_event_hub::EventHub {
            unsafe {
                if let Some(s) = &mut _EVENT_HUB {
                    s
                } else {
                    _EVENT_HUB = Some(ic_event_hub::EventHub::default());
                    get_event_hub()
                }
            }
        }

        #[allow(unused_must_use)]
        pub fn emit(_event: impl ic_event_hub::IEvent) {
            //union_utils::fns::log("ic_event_hub.emit()");

            let event = _event.to_event();
            let hub = get_event_hub();

            let listeners = hub.match_event_listeners_by_topics(&event.topics);
            if listeners.is_empty() {
                return;
            }

            let event_raw = ic_cdk::export::candid::encode_args((event.clone(),)).unwrap();

            for listener in listeners.iter() {
                ic_cdk::api::call::call_raw(
                    listener.canister_id,
                    listener.method_name.as_str(),
                    event_raw.clone(),
                    0,
                );
            }
        }
    }
}

#[macro_export]
macro_rules! implement_add_event_listeners {
    (guard = $guard:literal) => {
        #[ic_cdk_macros::update(guard = $guard)]
        fn add_event_listeners(
            payload: ic_event_hub::AddEventListenersPayload
        ) {
            //union_utils::fns::log("ic_event_hub.add_event_listeners()");

            let hub = get_event_hub();

            for listener in payload.listeners.into_iter() {
                hub.add_event_listener(
                    listener.filter,
                    listener.callback_method_name,
                    ic_cdk::caller(),
                );
            }
        }
    };
    () => {
        #[ic_cdk_macros::update]
        fn add_event_listeners(
            payload: ic_event_hub::AddEventListenersPayload
        ) {
            //union_utils::fns::log("ic_event_hub.add_event_listeners()");

            let hub = get_event_hub();

            for listener in payload.listeners.into_iter() {
                hub.add_event_listener(
                    listener.filter,
                    listener.callback_method_name,
                    ic_cdk::caller(),
                );
            }
        }
    }
}

#[macro_export]
macro_rules! implement_remove_event_listeners {
    (guard = $guard:literal) => {
        #[ic_cdk_macros::update(guard = $guard)]
        fn remove_event_listeners(
            payload: ic_event_hub::RemoveEventListenersPayload
        ) -> Vec<Result<(), String>> {
            //union_utils::fns::log("ic_event_hub.remove_event_listeners()");

            let hub = get_event_hub();
            let mut result = vec![];

            for f_m in payload.filters_and_method_names.into_iter() {
                result.push(hub.remove_event_listener(&f_m.filter, f_m.method_name, ic_cdk::caller()));
            }

            result
        }
    };
    () => {
        #[ic_cdk_macros::update]
        fn remove_event_listeners(
            payload: ic_event_hub::RemoveEventListenersPayload
        ) -> Vec<Result<(), String>> {
            //union_utils::fns::log("ic_event_hub.remove_event_listeners()");

            let hub = get_event_hub();
            let mut result = vec![];

            for f_m in payload.filters_and_method_names.into_iter() {
                result.push(hub.remove_event_listener(&f_m.filter, f_m.method_name, ic_cdk::caller()));
            }

            result
        }
    }
}

#[macro_export]
macro_rules! implement_get_event_listeners {
    (guard = $guard:literal) => {
        #[ic_cdk_macros::query(guard = $guard)]
        fn get_event_listeners(
            payload: ic_event_hub::GetEventListenersPayload
        ) -> Vec<Vec<ic_event_hub::RemoteCallEndpoint>> {
            //union_utils::fns::log("ic_event_hub.get_event_listeners()");

            let hub = get_event_hub();
            let mut res = vec![];

            for filter in payload.filters.iter() {
                res.push(hub.match_event_listeners(filter));
            }

            res
        }
    };
    () => {
        #[ic_cdk_macros::query]
        fn get_event_listeners(
            payload: ic_event_hub::GetEventListenersPayload
        ) -> Vec<Vec<ic_event_hub::RemoteCallEndpoint>> {
            //union_utils::fns::log("ic_event_hub.get_event_listeners()");

            let hub = get_event_hub();
            let mut res = vec![];

            for filter in payload.filters.iter() {
                res.push(hub.match_event_listeners(filter));
            }

            res
        }
    }
}