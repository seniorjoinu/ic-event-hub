use std::collections::{BTreeMap, BTreeSet, HashSet};

use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk::{caller, print};

pub mod api;

// ------------ CONSTS ------------------

pub const EVENT_NAME_FIELD: &str = "__event_name";

// ------------ TYPES -------------------

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

#[derive(CandidType, Deserialize)]
pub struct EventListenerExt {
    pub filter: EventFilter,
    pub endpoint: RemoteCallEndpoint,
}

// ---------- API TYPES ---------------

#[derive(CandidType, Deserialize)]
pub struct AddEventListenersRequest {
    pub listeners: Vec<EventListenerExt>,
}

pub type RemoveEventListenersRequest = AddEventListenersRequest;

#[derive(CandidType, Deserialize)]
pub struct RemoveEventListenersResponse {
    pub results: Vec<Result<(), String>>,
}

#[derive(CandidType, Deserialize)]
pub struct BecomeEventListenerRequest {
    pub listeners: Vec<EventListener>,
}

pub type StopBeingEventListenerRequest = BecomeEventListenerRequest;

#[derive(CandidType, Deserialize)]
pub struct StopBeingEventListenerResponse {
    pub results: Vec<Result<(), String>>,
}

#[derive(CandidType, Deserialize)]
pub struct GetEventListenersRequest {
    pub filters: Vec<EventFilter>,
}

#[derive(CandidType, Deserialize)]
pub struct GetEventListenersResponse {
    pub listeners: Vec<Vec<RemoteCallEndpoint>>,
}

// --------------- EVENT HUB --------------

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
            .filter(|&entry| entry.0 .0.is_subset(topics))
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
        let listeners = self
            .0
            .get_mut(filter)
            .ok_or_else(|| String::from("No such filter"))?;

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

// ------------------- UTILS ----------------------

pub fn log(msg: &str) {
    print(format!("[caller: {}]: {}", caller(), msg))
}
