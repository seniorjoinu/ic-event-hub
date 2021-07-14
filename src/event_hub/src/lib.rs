use std::collections::{BTreeMap, BTreeSet, HashSet};

use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

pub const EVENT_NAME_FIELD: &str = "__event_name";

#[derive(PartialEq, Eq, Hash, Clone, Debug, CandidType, Deserialize)]
pub struct RemoteCallEndpoint {
    pub canister_id: Principal,
    pub method_name: String,
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, CandidType, Deserialize)]
pub struct EventField {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Event {
    pub topics: BTreeSet<EventField>,
    pub values: Vec<EventField>,
}

pub trait IEvent {
    fn to_event(&self) -> Event;
    fn from_event(event: Event) -> Self;
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, CandidType, Deserialize)]
pub struct EventFilter(pub BTreeSet<EventField>);

pub trait IEventFilter {
    fn to_event_filter(&self) -> EventFilter;
    fn from_event_filter(filter: EventFilter) -> Self;
}

#[derive(CandidType, Deserialize)]
pub struct EventListener {
    pub filter: EventFilter,
    pub callback_method_name: String,
}

#[derive(Default)]
pub struct EventHub(BTreeMap<EventFilter, HashSet<RemoteCallEndpoint>>);

impl EventHub {
    pub fn add_event_listener(
        &mut self,
        filter: EventFilter,
        event_listener_method_name: String,
        caller: Principal,
    ) {
        let listener = RemoteCallEndpoint {
            canister_id: caller,
            method_name: event_listener_method_name,
        };

        let listeners = self.0.entry(filter).or_insert_with(HashSet::new);

        listeners.insert(listener);
    }

    pub fn match_event_listeners(&self, filter: &EventFilter) -> Vec<RemoteCallEndpoint> {
        self.match_event_listeners_by_topics(&filter.0)
    }

    pub fn match_event_listeners_by_topics(
        &self,
        topics: &BTreeSet<EventField>,
    ) -> Vec<RemoteCallEndpoint> {
        self.0
            .iter()
            .filter(|&entry| entry.0.0.is_subset(topics))
            .map(|entry| entry.1.clone())
            .flatten()
            .collect()
    }

    pub fn remove_event_listener(
        &mut self,
        filter: &EventFilter,
        event_listener_method_name: String,
        caller: Principal,
    ) -> Result<(), String> {
        let listeners = self.0
            .get_mut(filter)
            .ok_or(String::from("No such filter"))?;

        let listener_to_remove = RemoteCallEndpoint {
            canister_id: caller,
            method_name: event_listener_method_name,
        };

        let res = listeners.remove(&listener_to_remove);

        if !res {
            Err(String::from("No such listener in that filter"))
        } else {
            Ok(())
        }
    }
}

/*
   Attempts to create a subscription for events emitted from the [emitter] matching the [filter].
   When events are emitted, they are sent to the listener and execute [callback_name].

   Multiple 'listens' with the same parameters have no effect.
*/
pub async fn listen(
    emitter: Principal,
    _filter: impl IEventFilter,
    callback_name: String,
) -> Result<(), String> {
    let filter = _filter.to_event_filter();

    let res: CallResult<()> =
        ic_cdk::api::call::call(emitter, callback_name.as_str(), (filter, )).await;

    if let Err(err) = res {
        return Err(err.1);
    }

    Ok(())
}

/*
   Attempts to create multiple subscriptions for events emitted from the [emitter] matching each of the [filters].
   When events are emitted, they are sent to the listener and execute one of the [callback_names].
   [filters] and [callback_names] should be provided with respect to their order.
*/
pub async fn listen_many(
    emitter: Principal,
    filters: Vec<impl IEventFilter>,
    callback_names: Vec<String>,
) -> Result<(), String> {
    if filters.len() != callback_names.len() {
        return Err(String::from("There are not as many filters as callbacks"));
    }

    for (i, filter) in filters.into_iter().enumerate() {
        let cb = callback_names[i].as_str();
        let filter = filter.to_event_filter();

        let res: CallResult<()> = ic_cdk::api::call::call(
            emitter,
            cb,
            (filter, ),
        ).await;

        if let Err(err) = res {
            return Err(err.1);
        }
    }

    Ok(())
}

#[derive(CandidType, Deserialize)]
pub struct AddEventListenersPayload {
    pub listeners: Vec<EventListener>,
}

#[derive(CandidType, Deserialize)]
pub struct FilterAndMethodName {
    pub filter: EventFilter,
    pub method_name: String,
}

#[derive(CandidType, Deserialize)]
pub struct RemoveEventListenersPayload {
    pub filters_and_method_names: Vec<FilterAndMethodName>,
}

#[derive(CandidType, Deserialize)]
pub struct GetEventListenersPayload {
    pub filters: Vec<EventFilter>,
}

#[macro_export]
macro_rules! implement_event_emitter {
    () => {
        static mut _EVENT_HUB: Option<event_hub::EventHub> = None;

        pub fn get_event_hub() -> &'static mut event_hub::EventHub {
            unsafe {
                if let Some(s) = &mut _EVENT_HUB {
                    s
                } else {
                    _EVENT_HUB = Some(event_hub::EventHub::default());
                    get_event_hub()
                }
            }
        }

        #[allow(unused_must_use)]
        pub fn emit(_event: impl event_hub::IEvent) {
            //union_utils::fns::log("event_hub.emit()");

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
            payload: event_hub::AddEventListenersPayload
        ) {
            //union_utils::fns::log("event_hub.add_event_listeners()");

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
            payload: event_hub::AddEventListenersPayload
        ) {
            //union_utils::fns::log("event_hub.add_event_listeners()");

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
            payload: event_hub::RemoveEventListenersPayload
        ) -> Vec<Result<(), String>> {
            //union_utils::fns::log("event_hub.remove_event_listeners()");

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
            payload: event_hub::RemoveEventListenersPayload
        ) -> Vec<Result<(), String>> {
            //union_utils::fns::log("event_hub.remove_event_listeners()");

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
            payload: event_hub::GetEventListenersPayload
        ) -> Vec<Vec<event_hub::RemoteCallEndpoint>> {
            //union_utils::fns::log("event_hub.get_event_listeners()");

            let hub = get_event_hub();
            let mut res = vec![];

            for filter in payload.filters.iter() {
                res.push(hub.match_event_listeners(filter));
            }

            res
        }

        #[ic_cdk_macros::update(guard = stringify!($guard))]
        fn get_certified_event_listeners(
            payload: event_hub::GetEventListenersPayload
        ) -> Vec<Vec<event_hub::RemoteCallEndpoint>> {
            get_event_listeners(payload)
        }
    };
    () => {
        #[ic_cdk_macros::query]
        fn get_event_listeners(
            payload: event_hub::GetEventListenersPayload
        ) -> Vec<Vec<event_hub::RemoteCallEndpoint>> {
            //union_utils::fns::log("event_hub.get_event_listeners()");

            let hub = get_event_hub();
            let mut res = vec![];

            for filter in payload.filters.iter() {
                res.push(hub.match_event_listeners(filter));
            }

            res
        }

        #[ic_cdk_macros::update]
        fn get_certified_event_listeners(
            payload: event_hub::GetEventListenersPayload
        ) -> Vec<Vec<event_hub::RemoteCallEndpoint>> {
            get_event_listeners(payload)
        }
    }
}
