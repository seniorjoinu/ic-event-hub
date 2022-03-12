use ic_cdk::export::candid::{export_service, Principal};
use ic_cdk::{print, trap};
use ic_cdk_macros::{init, query, update};

use ic_event_hub::api::IEventHubClient;
use ic_event_hub::types::{CallbackInfo, EventFilter, SubscribeRequest};
use ic_event_hub::types::{Event, IEvent};
use ic_event_hub_macros::Event;

// ------------- MAIN LOGIC -------------------

pub struct RequestCounterMirror {
    pub emitter_canister_id: Principal,
    pub counter: u64,
    pub times_events_callback_triggered: u64,
}

#[derive(Event, Debug)]
pub struct MirrorEvent {
    pub payload: Vec<u8>,
}

#[init]
fn init(emitter_canister_id: Principal) {
    unsafe {
        STATE = Some(RequestCounterMirror {
            emitter_canister_id,
            counter: 0,
            times_events_callback_triggered: 0,
        });
    }
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

#[query]
fn get_counter_value() -> u64 {
    get_state().counter
}

#[query]
fn get_times_events_callback_triggered() -> u64 {
    get_state().times_events_callback_triggered
}

#[update]
fn events_callback(events: Vec<Event>) {
    print(format!("Received batched events: {:?}", events).as_str());
    get_state().times_events_callback_triggered += 1;

    for event in events {
        if event.get_name().as_str() == "MirrorEvent" {
            let ev: MirrorEvent = MirrorEvent::from_event(event);
            print(format!("Got event: {:?}", ev).as_str());

            get_state().counter += 1;
        }
    }
}

// ------------------ STATE ----------------------

static mut STATE: Option<RequestCounterMirror> = None;

pub fn get_state() -> &'static mut RequestCounterMirror {
    unsafe { STATE.as_mut().unwrap() }
}

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
