use std::collections::{BTreeMap, BTreeSet, HashSet};

use ic_cdk::export::Principal;
use union_utils::RemoteCallEndpoint;

use crate::types::{EventField, EventFilter};

/// A struct that associates event topics with subscribed listeners
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

    #[inline(always)]
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

#[cfg(test)]
mod tests {
    use union_utils::{random_principal_test, RemoteCallEndpoint};

    use crate::event_hub::EventHub;
    use crate::types::{EventField, EventFilter};

    #[test]
    fn main_flow_works_fine() {
        let mut event_hub = EventHub::default();

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
