use std::collections::BTreeSet;

use candid::{decode_one, CandidType as CandidTypeX, Deserialize};
use ic_cdk::export::candid::CandidType;
use ic_cdk::export::Principal;

use crate::EVENT_NAME_FIELD;

/// Serialized representation of some field of an event
#[derive(
    Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, CandidType, Deserialize, CandidTypeX,
)]
pub struct EventField {
    pub name: String,
    pub value: Vec<u8>,
}

/// Serialized event structure
#[derive(Clone, Debug, CandidType, Deserialize, CandidTypeX)]
pub struct Event {
    pub topics: BTreeSet<EventField>,
    pub values: Vec<EventField>,
}

impl Event {
    /// Finds a serialized name of the event struct, deserializes it and returns
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

/// Represents an struct that could be serialized into an `Event`
///
/// use `#[derive(Event)]` to implement it automatically
pub trait IEvent {
    fn to_event(&self) -> Event;
    fn from_event(event: Event) -> Self;
}

/// A set of topics of interest of a particular event listener
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, CandidType, Deserialize)]
pub struct EventFilter(pub BTreeSet<EventField>);

impl EventFilter {
    pub fn empty() -> Self {
        Self(BTreeSet::new())
    }
}

/// Represents a struct that could be serialized into an `EventFilter`
///
/// using `#[derive(Event)]` you're also generate such a filter automatically
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
pub struct BecomeEventListenerRequest {
    pub listeners: Vec<EventListener>,
}

pub type StopBeingEventListenerRequest = BecomeEventListenerRequest;

#[derive(CandidType, Deserialize)]
pub struct GetEventListenersRequest {
    pub filters: Vec<EventFilter>,
}

#[derive(CandidType, Deserialize)]
pub struct GetEventListenersResponse {
    pub listeners: Vec<Vec<RemoteCallEndpoint>>,
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, CandidType, Deserialize)]
pub struct RemoteCallEndpoint {
    pub canister_id: Principal,
    pub method_name: String,
}
