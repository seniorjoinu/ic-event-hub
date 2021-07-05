## IC Event hub

A library that lets you easily implement pub/sub capabilities for your canister

```rust
// counter.rs

use event_hub::implement_event_emitter;
use event_hub_macros::Event;
use ic_cdk::caller;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use ic_cdk_macros::update;

// Define some event (a piece of data, other canisters can listen to and react to) 
#[derive(Event, CandidType, Deserialize)]
struct StateChangedEvent {
    pub new_state: u64,
    #[topic]
    pub caller: Principal,
}

// use this function to automatically implement everything the protocol needs
// after this you'll be able to use 'emit()' function, which makes all the magic
implement_event_emitter!();

// let's define some state here
// this is a COUNTER example
static mut STATE: u64 = 0;

// add increment function
#[update]
fn inc() {
    unsafe {
        STATE += 1;
    }

    // emit your event
    // it will encode the event and pass it to every listener if there is any
    emit(StateChangedEvent {
        new_state: unsafe { STATE },
        caller: caller(),
    });
}
```

```rust
// counter-interaction-registry.rs

use event_hub_macros::{EventFilter, Event};
use event_hub::{IEvent, Event, listen};
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};
use std::collections::HashMap;
use ic_cdk_macros::{update, init, query};
use ic_cdk::caller;

// define the same event
#[derive(Event, CandidType, Deserialize)]
struct StateChangedEvent {
    pub new_state: u64,
    #[topic]
    pub caller: Principal
}

// define some event filter (a piece of data, which contains a topics to filter events by)
#[derive(EventFilter, CandidType, Deserialize)]
#[EventName = "StateChangedEvent"]
struct StateChangedEventFilter {
    pub caller: Principal
}

// this canister will count how many interactions each caller had with the counter canister
static mut STATE: Option<HashMap<Principal, u64>> = None;

#[init]
fn init() {
    unsafe {
        STATE = Some(HashMap::new())
    }
}

// let's give everyone an ability to subscribe to the counter canister
// and make this canister count their interactions for them
// you can do that by invoking 'listen' function
#[update]
#[allow(unused_must_use)]
async fn start_counting_changes_by_me() {
    listen(
        Principal::from_text("counter canister principal").unwrap(),
        StateChangedEventFilter {
            caller: caller()
        },
        String::from("count_changes_by_caller")
    ).await;
}

// this function will be executed by the counter canister
// when it emits an event
#[update]
fn count_changes_by_caller(ev: Event) {
    let event = StateChangedEvent::from_event(ev);

    let state = unsafe { STATE.as_mut().unwrap() };

    let count = state.get(&event.caller).cloned().unwrap_or(0);
    state.insert(event.caller, count + 1);
}

// this function let's everyone to check an amount of their interactions 
#[query]
fn how_many_changes_do_i_have() -> u64 {
    let state = unsafe { STATE.as_mut().unwrap() };

    state.get(&caller()).cloned().unwrap_or(0)
}
```

This way we successfully extended our counter canister from the outside.