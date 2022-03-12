use candid::ser::ValueSerializer;
use candid::CandidType;
use std::collections::{btree_map, hash_map, BinaryHeap};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ic_cdk::export::Principal;

use crate::types::{
    CallbackInfoExt, EncodedEventBatch, Event, EventField, EventFilter, EventHubError,
    RemoteCallEndpoint, TimestampedRemoteCallEndpoint,
};

/// A struct that associates event topics with subscribed listeners
pub struct EventHub {
    pub(crate) batch_making_duration_nano: u64,
    pub(crate) batch_max_size_bytes: usize,
    pub(crate) listeners: HashMap<EventFilter, HashSet<RemoteCallEndpoint>>,
    pub(crate) pending_batch: HashMap<RemoteCallEndpoint, EncodedEventBatch>,
    pub(crate) pending_batch_queue: BinaryHeap<TimestampedRemoteCallEndpoint>,
    pub(crate) ready_batches: BTreeMap<RemoteCallEndpoint, Vec<EncodedEventBatch>>,
}

impl EventHub {
    pub fn new(batch_making_duration_nano: u64, batch_max_size_bytes: usize) -> Self {
        EventHub {
            batch_making_duration_nano,
            batch_max_size_bytes,
            listeners: HashMap::default(),
            pending_batch: HashMap::default(),
            pending_batch_queue: BinaryHeap::new(),
            ready_batches: BTreeMap::default(),
        }
    }

    pub fn set_batch_making_duration_nano(&mut self, new_duration: u64) {
        self.batch_making_duration_nano = new_duration;
    }

    pub fn set_max_batch_size(&mut self, max: usize) {
        self.batch_max_size_bytes = max;
    }

    pub(crate) fn pop_pending_events(
        &mut self,
    ) -> Option<(RemoteCallEndpoint, Vec<EncodedEventBatch>)> {
        let (endpoint, _) = self.ready_batches.iter_mut().next_back()?;

        let endpoint = endpoint.clone();
        let events = self.ready_batches.remove(&endpoint).unwrap();

        Some((endpoint, events))
    }

    pub(crate) fn push_pending_event(
        &mut self,
        pending_event: Event,
        timestamp: u64,
    ) -> Result<(), EventHubError> {
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
                    let batch = EncodedEventBatch::new(event_value_ser.get_result(), timestamp);

                    e.insert(batch);

                    self.pending_batch_queue
                        .push(TimestampedRemoteCallEndpoint {
                            timestamp,
                            endpoint: listener.clone(),
                        });
                }
                hash_map::Entry::Occupied(mut e) => {
                    let batch = e.get_mut();
                    let total_size_bytes = batch.content.len() + event_value_ser.get_result().len();

                    if total_size_bytes <= self.batch_max_size_bytes {
                        batch.add_event(event_value_ser.get_result());
                    } else {
                        let new_batch =
                            EncodedEventBatch::new(event_value_ser.get_result(), timestamp);
                        let old_batch = e.insert(new_batch);

                        self.pending_batch_queue
                            .push(TimestampedRemoteCallEndpoint {
                                timestamp,
                                endpoint: listener.clone(),
                            });

                        self.add_ready_batch(listener, old_batch);
                    }
                }
            };
        }

        Ok(())
    }

    pub(crate) fn transform_pending_to_ready_by_time(&mut self, timestamp: u64) {
        loop {
            let cur_opt = self.pending_batch_queue.peek();
            if cur_opt.is_none() {
                break;
            }

            let cur = cur_opt.unwrap();

            if cur.timestamp + self.batch_making_duration_nano > timestamp {
                break;
            }

            let cur = self.pending_batch_queue.pop().unwrap();
            let batch = self.pending_batch.get(&cur.endpoint).unwrap();

            if batch.timestamp != cur.timestamp {
                continue;
            }

            let batch = self.pending_batch.remove(&cur.endpoint).unwrap();
            self.add_ready_batch(cur.endpoint, batch);
        }
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

    pub fn get_listeners(&self) -> &HashMap<EventFilter, HashSet<RemoteCallEndpoint>> {
        &self.listeners
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
        let mut event_hub = EventHub::new(0, 0);

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
