use candid::ser::ValueSerializer;
use candid::CandidType;
use std::collections::{btree_map, hash_map};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ic_cdk::export::Principal;
use ic_cdk::print;

use crate::types::{Event, EventField, EventFilter, RemoteCallEndpoint};

#[derive(Debug)]
pub enum EventHubError {
    EventHasNoActiveListeners,
    EventIsTooBig,
}

pub struct EncodedEventBatch {
    pub content: Vec<u8>,
    pub events_count: usize,
}

impl EncodedEventBatch {
    pub fn new(content: &[u8]) -> Self {
        Self {
            content: Vec::from(content),
            events_count: 1,
        }
    }

    pub fn add_event(&mut self, content: &[u8]) {
        self.content.extend_from_slice(content);
        self.events_count += 1;
    }
}

/// A struct that associates event topics with subscribed listeners
pub struct EventHub {
    pub batch_min_size_bytes: usize,
    pub batch_max_size_bytes: usize,
    pub listeners: HashMap<EventFilter, HashSet<RemoteCallEndpoint>>,
    pub pending_batch: HashMap<RemoteCallEndpoint, EncodedEventBatch>,
    pub ready_batches: BTreeMap<RemoteCallEndpoint, Vec<EncodedEventBatch>>,
}

impl EventHub {
    pub fn new() -> Self {
        EventHub {
            batch_min_size_bytes: 1 * 1024,
            batch_max_size_bytes: 300 * 1024,
            listeners: HashMap::default(),
            pending_batch: HashMap::default(),
            ready_batches: BTreeMap::default(),
        }
    }

    pub fn set_min_batch_size(&mut self, min: usize) {
        self.batch_min_size_bytes = min;
    }

    pub fn set_max_batch_size(&mut self, max: usize) {
        self.batch_max_size_bytes = max;
    }

    pub fn pop_pending_events(&mut self) -> Option<(RemoteCallEndpoint, Vec<EncodedEventBatch>)> {
        let (endpoint, _) = self.ready_batches.iter_mut().next_back()?;

        let endpoint = endpoint.clone();
        let events = self.ready_batches.remove(&endpoint).unwrap();

        Some((endpoint, events))
    }

    pub fn push_pending_event(&mut self, pending_event: Event) -> Result<(), EventHubError> {
        let listeners = self.match_event_listeners_by_topics(&pending_event.topics);

        if listeners.is_empty() {
            // when nobody listens to the event it is ignored
            return Err(EventHubError::EventHasNoActiveListeners);
        }

        let mut event_value_ser = ValueSerializer::new();
        pending_event
            .idl_serialize(&mut event_value_ser)
            .expect("Unable to serialize an event");

        if event_value_ser.get_result().len() >= self.batch_max_size_bytes {
            return Err(EventHubError::EventIsTooBig);
        }

        for listener in listeners {
            match self.pending_batch.entry(listener.clone()) {
                hash_map::Entry::Vacant(e) => {
                    let batch = EncodedEventBatch::new(event_value_ser.get_result());

                    // if the batch is already good to go - add it to ready_batches, otherwise, add to pending
                    if batch.content.len() >= self.batch_min_size_bytes {
                        self.add_ready_batch(listener, batch);
                    } else {
                        e.insert(batch);
                    }
                }
                hash_map::Entry::Occupied(mut e) => {
                    let batch = e.get_mut();
                    let total_size_bytes = batch.content.len() + event_value_ser.get_result().len();

                    print(format!("{}", total_size_bytes).as_str());

                    if total_size_bytes < self.batch_min_size_bytes {
                        batch.add_event(event_value_ser.get_result());
                    } else if total_size_bytes >= self.batch_min_size_bytes
                        && total_size_bytes <= self.batch_max_size_bytes
                    {
                        let mut batch = e.remove();
                        batch.add_event(event_value_ser.get_result());

                        self.add_ready_batch(listener, batch);
                    } else {
                        let new_batch = EncodedEventBatch::new(event_value_ser.get_result());
                        let old_batch = e.insert(new_batch);

                        self.add_ready_batch(listener, old_batch);
                    }
                }
            };
        }

        Ok(())
    }

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

        let listeners = self.listeners.entry(filter).or_insert_with(HashSet::new);

        listeners.insert(listener);
    }

    pub fn match_event_listeners(&self, filter: &EventFilter) -> Vec<RemoteCallEndpoint> {
        self.match_event_listeners_by_topics(&filter.0)
    }

    pub fn match_event_listeners_by_topics(
        &self,
        topics: &BTreeSet<EventField>,
    ) -> Vec<RemoteCallEndpoint> {
        self.listeners
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
            .listeners
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

    fn add_ready_batch(&mut self, listener: RemoteCallEndpoint, batch: EncodedEventBatch) {
        match self.ready_batches.entry(listener) {
            btree_map::Entry::Vacant(e) => {
                e.insert(vec![batch]);
            }
            btree_map::Entry::Occupied(mut e) => {
                e.get_mut().push(batch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event_hub::EventHub;
    use crate::types::{EventField, EventFilter, RemoteCallEndpoint};
    use candid::Principal;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random_principal_test() -> Principal {
        Principal::from_slice(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_be_bytes(),
        )
    }

    #[test]
    fn main_flow_works_fine() {
        let mut event_hub = EventHub::new();

        let field_1 = EventField {
            name: String::from("1"),
            value: vec![1],
        };
        let field_2 = EventField {
            name: String::from("2"),
            value: vec![2],
        };
        let field_3 = EventField {
            name: String::from("3"),
            value: vec![3],
        };

        let event_filter_1 = EventFilter(vec![field_1.clone()].into_iter().collect());
        let event_filter_2 = EventFilter(vec![field_2.clone()].into_iter().collect());
        let event_filter_3 = EventFilter(vec![field_3.clone()].into_iter().collect());
        let event_filter_1_2 =
            EventFilter(vec![field_1.clone(), field_2.clone()].into_iter().collect());
        let event_filter_1_3 =
            EventFilter(vec![field_1.clone(), field_3.clone()].into_iter().collect());
        let event_filter_2_3 =
            EventFilter(vec![field_2.clone(), field_3.clone()].into_iter().collect());
        let event_filter_1_2_3 = EventFilter(vec![field_1, field_2, field_3].into_iter().collect());

        let endpoint_1 = RemoteCallEndpoint {
            canister_id: random_principal_test(),
            method_name: String::from("test_1"),
        };
        let endpoint_2 = RemoteCallEndpoint {
            canister_id: random_principal_test(),
            method_name: String::from("test_2"),
        };
        let endpoint_3 = RemoteCallEndpoint {
            canister_id: random_principal_test(),
            method_name: String::from("test_3"),
        };

        event_hub.add_event_listener(
            event_filter_1.clone(),
            endpoint_1.method_name.clone(),
            endpoint_1.canister_id,
        );

        let endpoints = event_hub.match_event_listeners(&event_filter_1);
        assert!(
            endpoints.contains(&endpoint_1),
            "Should contain endpoint #1"
        );
        assert_eq!(endpoints.len(), 1, "Should match exactly 1 endpoint");

        let endpoints = event_hub.match_event_listeners(&event_filter_2);
        assert!(endpoints.is_empty(), "Should not match filter #2");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2);
        assert!(!endpoints.is_empty(), "Should match filter #1_2");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2_3);
        assert!(!endpoints.is_empty(), "Should match filter #1_2_3");

        event_hub.add_event_listener(
            event_filter_2.clone(),
            endpoint_2.method_name.clone(),
            endpoint_2.canister_id,
        );

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2);
        assert_eq!(endpoints.len(), 2, "Should match filter #1_2");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_3);
        assert_eq!(endpoints.len(), 1, "Should match filter #1_3");
        assert!(
            endpoints.contains(&endpoint_1),
            "Only endpoint #1 should match"
        );

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2_3);
        assert_eq!(endpoints.len(), 2, "Should match filter #1_2_3");

        event_hub
            .remove_event_listener(
                &event_filter_1.clone(),
                endpoint_1.method_name.clone(),
                endpoint_1.canister_id,
            )
            .ok()
            .unwrap();

        event_hub
            .remove_event_listener(
                &event_filter_2.clone(),
                endpoint_2.method_name.clone(),
                endpoint_2.canister_id,
            )
            .ok()
            .unwrap();

        event_hub.add_event_listener(
            event_filter_1_2_3.clone(),
            endpoint_3.method_name.clone(),
            endpoint_3.canister_id,
        );

        let endpoints = event_hub.match_event_listeners(&event_filter_1);
        assert!(endpoints.is_empty(), "Should not match filter #1");

        let endpoints = event_hub.match_event_listeners(&event_filter_2);
        assert!(endpoints.is_empty(), "Should not match filter #2");

        let endpoints = event_hub.match_event_listeners(&event_filter_3);
        assert!(endpoints.is_empty(), "Should not match filter #3");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2);
        assert!(endpoints.is_empty(), "Should not match filter #1_2");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_3);
        assert!(endpoints.is_empty(), "Should not match filter #1_3");

        let endpoints = event_hub.match_event_listeners(&event_filter_2_3);
        assert!(endpoints.is_empty(), "Should not match filter #2_3");

        let endpoints = event_hub.match_event_listeners(&event_filter_1_2_3);
        assert_eq!(endpoints.len(), 1, "Should match filter #1_2_3");
        assert!(endpoints.contains(&endpoint_3), "Should contain endpoint 3");
    }
}
