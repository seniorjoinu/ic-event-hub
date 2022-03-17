#[cfg(test)]
mod tests {
    use ic_event_hub::{implement_event_emitter, implement_subscribe, implement_unsubscribe};
    use ic_event_hub::types::{IEvent, IEventFilter};
    use ic_event_hub_macros::Event;

    implement_event_emitter!();
    implement_subscribe!(guard = "g");
    implement_unsubscribe!();

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
        let event_de = TestEvent::from_event(event_ser.clone());

        assert_eq!(event, event_de);
        assert_eq!(event_ser.get_name(), String::from("TestEvent"));
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
