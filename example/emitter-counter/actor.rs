use ic_cdk::caller;
use ic_cdk::export::candid::{export_service, Principal};
use ic_cdk_macros::{query, update};
use ic_event_hub_macros::{implement_become_event_listener, implement_event_emitter, Event};
use union_utils::log;

// ------------- MAIN LOGIC -------------------

#[derive(Default)]
pub struct Counter {
    pub counter: u64,
}

#[derive(Event)]
pub struct IncrementEvent {
    #[topic]
    pub by: Principal,
    pub current_value: u64,
}

#[update]
fn inc() -> u64 {
    log("emitter.inc()");

    let state = get_state();

    state.counter += 1;

    emit(IncrementEvent {
        by: caller(),
        current_value: state.counter,
    });

    state.counter
}

// ------------------ EVENT HUB ------------------

implement_event_emitter!();
implement_become_event_listener!();

// ------------------ STATE ----------------------

static mut STATE: Option<Counter> = None;

pub fn get_state() -> &'static mut Counter {
    unsafe {
        match STATE.as_mut() {
            Some(s) => s,
            None => {
                STATE = Some(Counter::default());
                get_state()
            }
        }
    }
}

// ---------------- CANDID -----------------------

export_service!();

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    __export_service()
}