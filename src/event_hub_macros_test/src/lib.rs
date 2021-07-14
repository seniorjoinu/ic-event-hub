#[cfg(test)]
mod tests {
    use event_hub::{
        IEvent,
        IEventFilter,
        implement_add_event_listeners,
        implement_event_emitter,
        implement_get_event_listeners,
        implement_remove_event_listeners,
    };
    use event_hub_macros::{Event, EventFilter};

    implement_event_emitter!();
    implement_get_event_listeners!();
    implement_add_event_listeners!();
    implement_remove_event_listeners!(guard = "g");

    fn g() -> Result<(), String> {
        Ok(())
    }

    #[derive(Event, Debug, PartialEq, Eq)]
    struct TestEvent {
        pub a: u8,
        #[topic]
        pub b: String,
    }

    #[derive(EventFilter, Debug, PartialEq, Eq)]
    #[EventName = "TestEvent"]
    struct TestEventFilter {
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
            b: String::from("kek"),
        };

        let filter_ser = filter.to_event_filter();
        let filter_de = TestEventFilter::from_event_filter(filter_ser);

        assert_eq!(filter, filter_de);
    }
}
