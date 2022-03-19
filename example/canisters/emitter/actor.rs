use ic_cdk::export::candid::export_service;
use ic_cdk_macros::{heartbeat, init, query, update};

use ic_event_hub::{implement_event_emitter, implement_subscribe, implement_unsubscribe};
use ic_event_hub_macros::Event;

// ------------- MAIN LOGIC -------------------

#[update]
fn mirror(data: Vec<u8>) {
    get_state().counter += 1;

    emit(MirrorEvent { data });
}

#[query]
fn get_requests_count() -> u64 {
    get_state().counter
}

// ------------------ EVENT HUB ------------------

#[derive(Event)]
pub struct MirrorEvent {
    pub data: Vec<u8>,
}

implement_event_emitter!(1_000_000_000 * 20, 500 * 1024);
implement_subscribe!();
implement_unsubscribe!();

#[heartbeat]
pub fn tick() {
    send_events();
}

// ------------------ STATE ----------------------

#[derive(Default)]
pub struct RequestCounter {
    pub counter: u64,
}

static mut STATE: Option<RequestCounter> = None;

pub fn get_state() -> &'static mut RequestCounter {
    unsafe { STATE.as_mut().unwrap() }
}

#[init]
fn init() {
    unsafe {
        STATE = Some(RequestCounter::default());
    }
}
