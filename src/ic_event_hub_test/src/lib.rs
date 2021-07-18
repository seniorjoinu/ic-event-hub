#[cfg(test)]
mod tests {
    use ic_event_hub::{
        implement_add_event_listeners, implement_become_event_listener, implement_event_emitter,
        implement_get_event_listeners, implement_remove_event_listeners,
        implement_stop_being_event_listener, IEvent, IEventFilter,
    };
    use ic_event_hub_macros::Event;

    implement_event_emitter!();
    implement_get_event_listeners!();
    implement_add_event_listeners!();
    implement_remove_event_listeners!(guard = "g");
    implement_become_event_listener!();
    implement_stop_being_event_listener!();

    fn g() -> Result<(), String> {
        Ok(())
    }

    #[derive(Event, Debug, PartialEq, Eq)]
    struct TestEvent {
        pub a: u8,
        #[topic]
        pub b: String,
    }

    #[test]
    fn events_serialization_works_fine() {
        let event = TestEvent {
            a: 10,
            b: String::from("kek"),
        };

        let event_ser = event.to_event();
        let event_de = TestEvent::from_event(event_ser);

        assert_eq!(event, event_de);
    }

    #[test]
    fn event_filters_serialization_works_fine() {
        let filter = TestEventFilter {
            b: Some(String::from("kek")),
        };

        let filter_ser = filter.to_event_filter();
        let filter_de = TestEventFilter::from_event_filter(filter_ser);

        assert_eq!(filter.b, filter_de.b);
    }
}
