use std::collections::BTreeSet;

use ic_cdk::export::candid::{decode_one, CandidType, Deserialize};
use union_utils::RemoteCallEndpoint;

use crate::EVENT_NAME_FIELD;

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

impl Event {
    #[inline(always)]
    pub fn get_name(&self) -> String {
        let encoded_name = self
            .topics
            .iter()
            .find(|&field| field.name == EVENT_NAME_FIELD)
            .cloned()
            .unwrap()
            .value;

        decode_one::<String>(encoded_name.as_slice()).unwrap()
    }
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
