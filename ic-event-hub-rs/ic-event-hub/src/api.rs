use candid::{CandidType, Deserialize};
use ic_cdk::api::call::CallResult;
use ic_cdk::call;
use ic_cdk::export::candid::Principal;

use crate::types::{
    AddEventListenersRequest, BecomeEventListenerRequest, GetEventListenersRequest,
    GetEventListenersResponse, RemoveEventListenersRequest, StopBeingEventListenerRequest,
};

/// Struct used to interact with event-emitter canisters
///
/// Usage:
/// ```
/// use ic_event_hub::types::{BecomeEventListenerRequest, EventListener};
/// use ic_event_hub::api::EventHubClient;
///
/// // create a new client obj
/// let client = EventHubClient::new(emitter_principal_id);
///
/// // call a function to become an event listener
/// client._become_event_listener(BecomeEventListenerRequest {
///     listeners: vec![
///         EventListener {
///             filter: ev_filter.to_event_filter(),
///             callback_method_name: String::from("_event_callback"),
///         },
///     ],
/// }).await?;
/// ```
/// ```
/// // define an event listener callback
/// #[ic_cdk_macros::update]
/// fn _event_callback(event: ic_event_hub::types::Event) {
///     ...
/// }
/// ```
#[derive(CandidType, Deserialize)]
pub struct EventHubClient {
    /// The principal ID of the canister to interact to
    pub canister_id: Principal,
}

impl EventHubClient {
    #[inline(always)]
    pub fn new(canister_id: Principal) -> Self {
        Self { canister_id }
    }

    /// Adds event listeners to the event-emitter subscription.
    ///
    /// Can only be called when `implement_add_event_listeners!()` is used on the event-emitter.
    #[inline(always)]
    pub async fn _add_event_listeners(&self, request: AddEventListenersRequest) -> CallResult<()> {
        call(self.canister_id, "_add_event_listeners", (request,)).await
    }

    /// Removes event listeners from the event-emitter subscription
    ///
    /// Can only be called when `implement_remove_event_listeners!()` is used on the event-emitter.
    /// The result could be `Err` in case there is no such listeners.
    #[inline(always)]
    pub async fn _remove_event_listeners(
        &self,
        request: RemoveEventListenersRequest,
    ) -> CallResult<()> {
        call(self.canister_id, "_remove_event_listeners", (request,)).await
    }

    /// Adds itself to the event-emitter subscription
    ///
    /// Can only be called when `implement_become_event_listener!()` is used on the event-emitter.
    #[inline(always)]
    pub async fn _become_event_listener(
        &self,
        payload: BecomeEventListenerRequest,
    ) -> CallResult<()> {
        call(self.canister_id, "_become_event_listener", (payload,)).await
    }

    /// Removes itself from the event-emitter subscription
    ///
    /// Can only be called when `implement_stop_being_event_listener!()` is used on the event-emitter.
    #[inline(always)]
    pub async fn _stop_being_event_listener(
        &self,
        request: StopBeingEventListenerRequest,
    ) -> CallResult<()> {
        call(self.canister_id, "_stop_being_event_listener", (request,)).await
    }

    /// Returns a list of all listeners subscribed to the event-emitter
    ///
    /// Can only be called when `implement_get_event_listeners!()' is used on the event-emitter.
    #[inline(always)]
    pub async fn _get_event_listeners(
        &self,
        request: GetEventListenersRequest,
    ) -> CallResult<(GetEventListenersResponse,)> {
        call(self.canister_id, "_get_event_listeners", (request,)).await
    }
}
