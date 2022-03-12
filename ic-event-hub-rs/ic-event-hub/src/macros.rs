#[macro_export]
macro_rules! implement_event_emitter {
    ($duration:expr, $max_size:expr) => {
        static mut _EVENT_HUB: Option<ic_event_hub::event_hub::EventHub> = None;

        pub fn get_event_hub() -> &'static mut ic_event_hub::event_hub::EventHub {
            unsafe {
                if let Some(s) = &mut _EVENT_HUB {
                    s
                } else {
                    _EVENT_HUB = Some(ic_event_hub::event_hub::EventHub::new($duration, $max_size));
                    get_event_hub()
                }
            }
        }

        pub fn emit(event: impl ic_event_hub::types::IEvent) {
            ic_event_hub::fns::emit_impl(event, get_event_hub());
        }

        pub fn send_events() {
            ic_event_hub::fns::send_events_impl(get_event_hub());
        }
    };
}

#[macro_export]
macro_rules! implement_subscribe {
    () => {
        #[ic_cdk_macros::update]
        fn subscribe(req: ic_event_hub::types::SubscribeRequest) {
            ic_event_hub::fns::subscribe_impl(req, get_event_hub());
        }
    };

    (guard = $guard:expr) => {
        #[ic_cdk_macros::update(guard = $guard)]
        fn subscribe(req: ic_event_hub::types::SubscribeRequest) {
            ic_event_hub::fns::subscribe_impl(req, get_event_hub());
        }
    };
}

#[macro_export]
macro_rules! implement_unsubscribe {
    () => {
        #[ic_cdk_macros::update]
        fn unsubscribe(req: ic_event_hub::types::UnsubscribeRequest) {
            ic_event_hub::fns::unsubscribe_impl(req, get_event_hub());
        }
    };

    (guard = $guard:expr) => {
        #[ic_cdk_macros::update(guard = $guard)]
        fn unsubscribe(req: ic_event_hub::types::UnsubscribeRequest) {
            ic_event_hub::fns::unsubscribe_impl(req, get_event_hub());
        }
    };
}
