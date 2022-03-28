use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::storage::{stable_restore, stable_save};
use ic_cdk_macros::{heartbeat, init, post_upgrade, pre_upgrade, query, update};

use ic_event_hub::event_hub::EventHub;
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

implement_event_emitter!(1_000_000_000 * 25, 1024 * 1024);
implement_subscribe!();
implement_unsubscribe!();

#[heartbeat]
pub fn tick() {
    send_events();
}

// ------------------ STATE ----------------------

#[derive(Default, CandidType, Deserialize)]
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

#[pre_upgrade]
fn pre_upgrade_hook() {
    let canister_state = unsafe { STATE.take() };
    let event_hub_state = _take_event_hub_state();

    stable_save((canister_state, event_hub_state)).expect("Unable to stable save");
}

#[post_upgrade]
fn post_upgrade_hook() {
    let (canister_state, event_hub_state): (Option<RequestCounter>, Option<EventHub>) =
        stable_restore().expect("Unable to stable restore");

    unsafe { STATE = canister_state };
    _put_event_hub_state(event_hub_state);
}
