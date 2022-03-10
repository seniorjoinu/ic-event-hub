use ic_cdk::export::candid::{export_service, Principal};
use ic_cdk::{caller, id, print};
use ic_cdk_macros::{heartbeat, init, query, update};

use ic_event_hub_macros::{implement_become_event_listener, implement_event_emitter, Event};

// ------------- MAIN LOGIC -------------------

#[derive(Default)]
pub struct RequestCounter {
    pub counter: u64,
}

#[derive(Event)]
pub struct MirrorEvent {
    pub payload: Vec<u8>,
}

#[update]
fn mirror() {
    get_state().counter += 1;

    emit(MirrorEvent {
        payload: vec![1; 100],
    });
}

#[query]
fn get_counter_value() -> u64 {
    get_state().counter
}

#[heartbeat]
pub fn tick() {
    send_events();
}

// ------------------ EVENT HUB ------------------

implement_event_emitter!();
implement_become_event_listener!();

// ------------------ STATE ----------------------

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

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}
