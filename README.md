## IC Event Hub

A rust library which applies IoC to your pub/sub

### Motivation

The main idea behind the Open Internet Services concept is ___collaboration___. The easier it is for us to integrate
canisters, the more fantastic collaborations we could achieve, the closer we are to the Open Internet.

This library greatly simplifies canister integration by flipping integration surface upside down. It enables your
canister to express its interfaces in terms of emitted events. Sending a message, your canister does not need to know a
signature of the remote canister anymore. Instead, the remote canister should know what events it wants to receive from
your canister. This enables us to create emitter canisters - ones which emit events, when something important happens
inside it, and which does not care who are the exact recipients of these events.

On the other side, listener-canisters (recipients) are able to make precise decisions which event topics they want to
receive updates on, which is good for both: performance/price and overall application logic.

### Tutorial

#### Task definition

Let's imagine the following example use-case:

* there is the "Coin Flipper" canister (next Flipper), which provides its users with a method to flip a coin using IC's
  secure random
* there is the "Lucky People Fund" canister (next Fund), that mints its own token to everyone who is lucky enough to
  flip heads three times in a row
* there is also the "Joinu's Coin Flipper Ledger" canister (next Ledger), which collects the history of only those coin
  flips made by Joinu's principal

Our goal is to make them all work together, only providing them with each others canister-id and an event structure,
nothing more. No sharing of function signatures or even function names.

#### Events and event filters

Everything starts off with the definition of events. We want Flipper to emit an event each time someone flips a coin.
This event should contain some `nat` ID, flip result - `Heads`/`Tails`, and the `principal`
of the user flipped the coin. Let's define it!

```rust
#[derive[ic_event_hub_macros::Event]]
pub struct FlipEvent {
    pub id: u64,
    pub heads: bool,
    #[topic]
    pub flipper: Principal,
}
```

This `FlipEvent` definition looks pretty normal for rust, the only interesting things are
annotations: `#[derive(Event)]`
and `#[topic]`. The first one is a derive macro, which will:

1. generate an implementation for `ic_event_hub::IEvent` trait, which we need in order to serialize this event in a way
   that is optimized for topic indexing; `#[topic]` annotation is used exactly for that - to mark fields, which could be
   used by a listener to specify what updates does it want to receive precisely
2. generate a `FlipEventFilter` struct for our `FlipEvent`, that makes it easier for listeners to use when they
   subscribe for updates.

Precisely the macro will expand into this code:

```rust
pub struct FlipEvent {
    pub id: u64,
    pub heads: bool,
    #[topic]
    pub flipper: Principal,
}

impl ic_event_hub::types::IEvent for FlipEvent {
    fn to_event(&self) -> ic_event_hub::Event {
        ...
    }
    
    fn from_event(event: ic_event_hub::types::Event) -> Self {
        ...
    }
}

// take a look at this struct
// notice it hase the same name as our `FlipEvent` struct, but with "Filter" postfix
pub struct FlipEventFilter {
    // also notice the type of the only topic we had in `FlipEvent` 
    // now transformed into optional - passing here `None` will create us a filter that will match ANY flip event
    pub flipper: Option<Principal>,
}

impl ic_event_hub::types::IEventFilter for FlipEventFilter {
    fn to_event_filter(&self) -> ic_event_hub::types::EventFilter {
        ...
    }
    
    fn from_event_filter(filter: ic_event_hub::types::EventFilter) -> Self {
        ...
    }
}
```

Now we have two structs `FlipEvent` and `FlipEventFilter` which will help us later.

#### How to become an event emitter

We have everything ready to implement our Flipper canister, which is the __emitter__ in terms of event-hub. Let's start
by implementing basic event-hub functionality:

```rust
// flipper-canister.rs

ic_event_hub_macros::implement_event_emitter!();
```

That's it. All we have to do is to call this macro which will automatically implement everything our canister needs to
emit events. Specifically it will generate the next functions:

1. `get_event_hub() -> ic_event_hub::event_hub::EventHub` - returns an object holding all the info about current active
   listeners; don't worry, you won't need it until you decide to extend `ic-event-hub` with some new functionality
2. `emit(event: ic_event_hub::types::IEvent)` - serializes the passed event and sends it to every single listener
   interested in it; __this function is what we'll mostly use__

Okay, now we can emit events, but how exactly does one become an event listener? There are several approaches to achieve
that: permissionless one, permissioned one and their combinations.

##### Permissioned way

If you want full control over the list of listener-canisters which should receive events from the emitter, you can call
these couple of macro:

```rust
// flipper-canister.rs

// notice optional "guard" parameter - here you pass a function that would check callers access rights for that operation
// it uses the same "guard" function format as `ic_cdk`
ic_event_hub_macros::implement_add_event_listener!(guard = "event_listener_guard");
ic_event_hub_macros::implement_remove_event_listener!(guard = "event_listener_guard");
```

which would generate these following endpoints for your emitter-canister, which you can use to manually add canisters
into listeners list and specify exactly what events they are able to receive:

```rust
#[ic_cdk_macros::update(guard = "event_listener_guard")]
fn _add_event_listeners(request: ic_event_hub::types::AddEventListenersRequest) {
    ...
}

#[ic_cdk_macros::update(guard = "event_listener_guard")]
fn _remove_event_listeners(request: ic_event_hub::types::RemoveEventListenersRequest) -> ic_event_hub::types::RemoveEventListenersResponse {
    ...
}
```

> For these endpoints you might need to define candid, so you could interact with them from the browser. You can find
> it [here](./ic-event-hub-rs/ic-event-hub-macros/can.did)

##### Permissionless way

But if you're fine with canisters to freely subscribe to and unsubscribe from your emitter canister's events you may
want to give them a convenient API for such an operation. The following macros will do that for you:

```rust
// flipper-canister.rs

// these macros also support "guard" functions - you could use them to, for example, restrict listener-canisters
// to fit some criteria you want
ic_event_hub_macros::implement_become_event_listener!();
ic_event_hub_macros::implement_stop_being_event_listener!();
```

which would generate these following endpoints for your emitter-canister, which other canisters could use to subscribe
or unsubscribe to the events your canister emits:

```rust
#[ic_cdk_macros::update]
fn _become_event_listener(request: ic_event_hub::types::BecomeEventListenerRequest) {
    ...
}

#[ic_cdk_macros::update]
fn _stop_being_event_listener(request: ic_event_hub::types::StopBeingEventListenerRequest) -> ic_event_hub::types::StopBeingEventListenerResponse {
    ...
}
```

> You don't need to modify your candid file for these endpoints. Later you'll see why.

##### Combinations

Experiment with it! Since each endpoint is represented by a separate single macro, you can choose which one you want
your canister to have. There is a lot of different combinations, considering usage of "guard" functions.

#### Emitting events

For the sake of this tutorial, let's imagine we've chosen the __permissionless__ way of adding new event-listeners.

Now, we have everything ready to start emitting events. This is the rough code our Flipper canister would have after
all:

```rust
// flipper-canister.rs

ic_event_hub_macros::implement_event_emitter!();
ic_event_hub_macros::implement_become_event_listener!();
ic_event_hub_macros::implement_stop_being_event_listener!();

#[derive[ic_event_hub_macros::Event]]
pub struct FlipEvent {
    pub id: u64,
    pub heads: bool,
    #[topic]
    pub flipper: Principal,
}

#[ic_cdk_macros::update]
async fn flip_a_coin() -> bool {
    let flip_result: bool = _get_random_flip().await;
    
    emit(FlipEvent {
        id: _generate_flip_id(),
        heads: flip_result,
        flipper: ic_cdk::caller(),
    });
    
    flip_result
}
```

That's it! As you can see, our emitter code does not care at all who will catch events it emits.

#### Catching events

Let's start with the Fund canister. As we defined, this canister needs to react to every coin flip, that happens inside
the Flipper canister, and mint some tokens to the most lucky coin flippers.

The Fund canister need to know the canister-id of the Flipper canister as well as the event it wants to listen to. Then
it could use this knowledge to subscribe to the Flipper canister and react to those events. To achieve that it could use
[canister client](./ic-event-hub-rs/ic-event-hub/src/api.rs) for event-hub listeners! It is a very simple, but helpful
rust struct with impl that gives us a strict API to interact with such an emitter-canister.

Since the logic is pretty straightforward now, let's roughly define how the Fund canister could look like:

```rust
// fund-canister.rs

// this struct could be shared through a library
#[derive[ic_event_hub_macros::Event]]
pub struct FlipEvent {
    pub id: u64,
    pub heads: bool,
    #[topic]
    pub flipper: Principal,
}

// this function should be called by Fund canister's controller from the outside
// at the moment they want their canister to start catching events
#[ic_cdk_macros::update(guard = "controller_guard")]
async fn start_listening_to_flipper_events() {
    let flipper_client = ic_event_hub::api::EventHubClient::new(_get_flipper_canister_id());
    let flip_event_filter = FlipEventFilter {
        flipper: None,
    };
    
    flipper_client._become_event_listener(BecomeEventListenerRequest {
        listeners: vec![
            EventListener {
                filter: (flip_event_filter as ic_event_hub::types::IEventFilter).to_event_filter(),
                
                // here we have to specify a "handler endpoint" which will be used to catch events
                // it's up to you to catch them into a single endpoint
                // or to create a separate endpoint for each event type 
                callback_method_name: String::from("_handle_flipper_event"),
            }
        ],
    })
        .await;
}

// you should check handler endpoint, so it could be called only by the canister you're listening to
#[ic_cdk_macros::update(guard = "flipper_guard")]
async fn _handle_flipper_event(event: ic_event_hub::types::Event) {
    // check the event name and if it's good - react the way you like
    if event.get_name().as_str() == "FlipEvent" {
        let ev: FlipEvent = FlipEvent::from_event(event);
    
        if _won_3_times_in_a_row(ev.flipper) {
            _mint_tokens(ev.flipper, 100).await;
        }
    }
}
```

Let's quickly recap what we just wrote.

1. If the emitter is in the permissionless mode, your listener-canister should somehow subscribe to events it wants to
   listen to. Since it can't do it automatically on `#[init]` (you can't send messages during init function), you'll
   have to do it manually.
2. While subscribing, listener have to specify a handler endpoint to which it wants to receive emitted events. These
   endpoints could be different for each subscription.
3. Handler endpoint can have any name and can have any return type (it is ignored by the emitter, anyway). The only
   restriction here - it should accept a single argument of type `ic_event_hub::types::Event`.

As you might guess, the implementation for the Ledger canister, will look very similar (especially with the degree of
detalization I've chosen for this tutorial). The only difference here is that the Ledger canister only wants to receive
events related to Joinu's principal. Let's imagine, Joinu has multiple public principals, luckiness each of which the
Ledger canister could keep track of.

Let's implement this scenario:

```rust
// ledger-canister.rs

// again, we need to know this struct beforehand
#[derive[ic_event_hub_macros::Event]]
pub struct FlipEvent {
    pub id: u64,
    pub heads: bool,
    #[topic]
    pub flipper: Principal,
}

// this is almost the same method that we had in the Fund canister, but here we want to create multiple subscriptions
#[ic_cdk_macros::update(guard = "controller_guard")]
async fn start_listening_to_flipper_events() {
    let flipper_client = ic_event_hub::api::EventHubClient::new(_get_flipper_canister_id());
    
    for joinu_principal in _get_joinu_principals() {
        let flip_event_filter = FlipEventFilter {
            flipper: Some(joinu_principal),
        };

        flipper_client._become_event_listener(BecomeEventListenerRequest {
            listeners: vec![
                EventListener {
                    filter: (flip_event_filter as ic_event_hub::types::IEventFilter).to_event_filter(),
                    callback_method_name: String::from("_handle_flipper_event"),
                }
            ],
        })
            .await;
    }
}

// again, don't forget to check the caller
#[ic_cdk_macros::update(guard = "flipper_guard")]
async fn _handle_flipper_event(event: ic_event_hub::types::Event) {
    if event.get_name().as_str() == "FlipEvent" {
        // here we will now only receive events of flips made by Joinu!
        let ev: FlipEvent = FlipEvent::from_event(event);
    
        _store_history_entry(ev);
    }
}
```

> You're not restricted on the number of topics your event could have. It could not have any topics at all - in that case every listener would catch every event of that event type.

As you might already guess, supplying multiple filters of the same event is an equivalent to logical __OR__. Contrary,
supplying a single filter with multiple topics set to `Some()` is an equivalent to logical __AND__.

#### Final notes

While the example used in this tutorial is purely synthetic and the implementation is abstract, I hope it was useful for
you. We reached our goal and went through almost everything this library has to offer.

Check the [example project](./example) for working code sample.

### Installation

```toml
# Cargo.toml

[dependencies]
ic-event-hub = "0.1.8"
ic-event-hub-macros = "0.1.8"
```

### API

Check crates.io

### Limitations

Right now `ic-event-hub` doesn't support canister upgrades, so all your queued tasks will be lost. This is due to a
limitation in `ic-cdk`, which doesn't support multiple stable variables at this moment. Once they do, I'll update this
library, so it will handle canister upgrades gracefully.

If you really want this functionality right now, you may try to serialize the state manually using `get_event_hub()`
function.

### Planned improvements

Once `heartbeat` IC functionality is ready, I have a plan to optimize messaging layer (with the help
of [ic-cron](https://github.com/seniorjoinu/ic-cron)), putting messages into a buffer and then bulk-sending them when
the heart beats. It should greatly reduce costs in scenarios when there are lots of events targeted to a single
listener.

### Candid

Depending on your decisions, you might want to extend your `.did` file with
the [following data](./ic-event-hub-rs/ic-event-hub-macros/can.did)

### Contribution

You can reach me out here on github opening an issue, or you could start a thread on dfinity's developer forum.

You're also welcome to suggest new features and open PR's.