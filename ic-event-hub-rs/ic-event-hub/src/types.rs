use std::cmp::{max, min, Ordering};
use std::collections::BTreeSet;

use candid::{decode_one, CandidType, Deserialize};
use ic_cdk::export::Principal;

use crate::EVENT_NAME_FIELD;

/// Serialized representation of some field of an event
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Debug, CandidType, Deserialize)]
pub struct EventField {
    pub name: String,
    pub value: Vec<u8>,
}

/// Serialized event structure
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Event {
    pub topics: BTreeSet<EventField>,
    pub values: Vec<EventField>,
}

impl Event {
    /// Finds a serialized name of the event struct, deserializes it and returns
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
pub struct CallbackInfo {
    pub filter: EventFilter,
    pub method_name: String,
}

#[derive(CandidType, Deserialize)]
pub struct CallbackInfoExt {
    pub filter: EventFilter,
    pub endpoint: RemoteCallEndpoint,
}

#[derive(Debug)]
pub enum EventHubError {
    EventHasNoActiveListeners,
    EventIsTooBig,
}

pub struct EncodedEventBatch {
    pub content: Vec<u8>,
    pub events_count: usize,
    pub timestamp: u64,
}

impl EncodedEventBatch {
    pub fn new(content: &[u8], timestamp: u64) -> Self {
        Self {
            content: Vec::from(content),
            events_count: 1,
            timestamp,
        }
    }

    pub fn add_event(&mut self, content: &[u8]) {
        self.content.extend_from_slice(content);
        self.events_count += 1;
    }
}

#[derive(Eq)]
pub struct TimestampedRemoteCallEndpoint {
    pub timestamp: u64,
    pub endpoint: RemoteCallEndpoint,
}

impl PartialEq for TimestampedRemoteCallEndpoint {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp) && self.endpoint.eq(&other.endpoint)
    }
}

impl PartialOrd for TimestampedRemoteCallEndpoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.timestamp
            .partial_cmp(&other.timestamp)
            .map(|it| it.reverse())
    }

    fn lt(&self, other: &Self) -> bool {
        self.timestamp.gt(&other.timestamp)
    }

    fn le(&self, other: &Self) -> bool {
        self.timestamp.ge(&other.timestamp)
    }

    fn gt(&self, other: &Self) -> bool {
        self.timestamp.lt(&other.timestamp)
    }

    fn ge(&self, other: &Self) -> bool {
        self.timestamp.le(&other.timestamp)
    }
}

impl Ord for TimestampedRemoteCallEndpoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp).reverse()
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        max(self, other)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        min(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        if self.timestamp < max.timestamp {
            max
        } else if self.timestamp > min.timestamp {
            min
        } else {
            self
        }
    }
}

// ---------- API TYPES ---------------

#[derive(CandidType, Deserialize)]
pub struct SubscribeRequest {
    pub callbacks: Vec<CallbackInfo>,
}

pub type UnsubscribeRequest = SubscribeRequest;

#[derive(CandidType, Deserialize)]
pub struct GetSubscribersRequest {
    pub filters: Vec<EventFilter>,
}

#[derive(CandidType, Deserialize)]
pub struct GetSubscribersResponse {
    pub subscribers: Vec<Vec<RemoteCallEndpoint>>,
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug, CandidType, Deserialize)]
pub struct RemoteCallEndpoint {
    pub canister_id: Principal,
    pub method_name: String,
}
