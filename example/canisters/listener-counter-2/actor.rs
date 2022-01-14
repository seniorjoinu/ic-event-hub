use ic_cdk::export::candid::{export_service, Principal};
use ic_cdk::{caller, trap};
use ic_cdk_macros::{init, query, update};

use ic_event_hub::api::EventHubClient;
use ic_event_hub::types::{BecomeEventListenerRequest, EventListener};
use ic_event_hub::types::{Event, IEvent, IEventFilter};
use ic_event_hub_macros::Event;

// ------------- MAIN LOGIC -------------------

pub struct FilteringCounterMirror {
    pub emitter_canister_id: Principal,
    pub counter: u64,
}

#[derive(Event)]
pub struct IncrementEvent {
    #[topic]
    pub by: Principal,
    pub current_value: u64,
}

#[init]
fn init(emitter_canister_id: Principal) {
    unsafe {
        STATE = Some(FilteringCounterMirror {
            emitter_canister_id,
            counter: 0,
        });
    }
}

#[update]
async fn start_listening() {
    let client = EventHubClient::new(get_state().emitter_canister_id);

    let filter = IncrementEventFilter { by: Some(caller()) };

    client
        ._become_event_listener(BecomeEventListenerRequest {
            listeners: vec![EventListener {
                filter: filter.to_event_filter(),
                callback_method_name: String::from("events_callback"),
            }],
        })
        .await
        .ok()
        .unwrap();
}

#[query]
fn get_counter_value() -> u64 {
    get_state().counter
}

#[update]
fn events_callback(events: Vec<Event>) {
    for event in events {
        if event.get_name().as_str() == "IncrementEvent" {
            let ev: IncrementEvent = IncrementEvent::from_event(event);
            get_state().counter = ev.current_value;
        }
    }
}

// ------------------ STATE ----------------------

static mut STATE: Option<FilteringCounterMirror> = None;

pub fn get_state() -> &'static mut FilteringCounterMirror {
    unsafe {
        match STATE.as_mut() {
            Some(s) => s,
            None => trap("No state found"),
        }
    }
}

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
