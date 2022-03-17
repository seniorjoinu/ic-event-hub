use ic_cdk::export::candid::export_service;
use ic_cdk_macros::{heartbeat, init, query, update};

use ic_event_hub::{implement_event_emitter, implement_subscribe};
use ic_event_hub_macros::Event;

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

implement_event_emitter!(1_000_000_000 * 20, 500 * 1024);
implement_subscribe!();

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
