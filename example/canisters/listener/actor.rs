use ic_cdk::export::candid::{export_service, Principal};
use ic_cdk::{print, trap};
use ic_cdk_macros::{init, query, update};

use ic_event_hub::api::IEventHubClient;
use ic_event_hub::types::{CallbackInfo, EventFilter, SubscribeRequest};
use ic_event_hub::types::{Event, IEvent};
use ic_event_hub_macros::Event;

// ------------- MAIN LOGIC -------------------

#[query]
fn get_events_received() -> u64 {
    get_state().events_received
}

#[query]
fn get_batches_received() -> u64 {
    get_state().batches_received
}

// ----------------- EVENT HUB ----------------------

#[derive(Event, Debug)]
pub struct MirrorEvent {
    pub data: Vec<u8>,
}

#[update]
async fn start_listening() {
    get_state()
        .emitter_canister_id
        .subscribe(SubscribeRequest {
            callbacks: vec![CallbackInfo {
                filter: EventFilter::empty(),
                method_name: String::from("events_callback"),
            }],
        })
        .await
        .ok()
        .unwrap();
}

#[update]
fn events_callback(events: Vec<Event>) {
    get_state().batches_received += 1;

    for event in events {
        if event.get_name().as_str() == "MirrorEvent" {
            let ev: MirrorEvent = MirrorEvent::from_event(event);
            print(format!("Got event: {:?}", ev).as_str());

            get_state().events_received += 1;
        }
    }
}

// ------------------ STATE ----------------------

pub struct RequestCounterMirror {
    pub emitter_canister_id: Principal,
    pub events_received: u64,
    pub batches_received: u64,
}

static mut STATE: Option<RequestCounterMirror> = None;

pub fn get_state() -> &'static mut RequestCounterMirror {
    unsafe { STATE.as_mut().unwrap() }
}

#[init]
fn init(emitter_canister_id: Principal) {
    unsafe {
        STATE = Some(RequestCounterMirror {
            emitter_canister_id,
            events_received: 0,
            batches_received: 0,
        });
    }
}
