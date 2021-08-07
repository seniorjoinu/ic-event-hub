use ic_cdk::api::call::CallResult;
use ic_cdk::call;
use ic_cdk::export::candid::{CandidType, Deserialize, Principal};

use crate::{
    AddEventListenersRequest, BecomeEventListenerRequest, GetEventListenersRequest,
    GetEventListenersResponse, RemoveEventListenersRequest, RemoveEventListenersResponse,
    StopBeingEventListenerRequest, StopBeingEventListenerResponse,
};

#[derive(CandidType, Deserialize)]
pub struct EventHubClient {
    pub canister_id: Principal,
}

impl EventHubClient {
    #[inline(always)]
    pub fn new(canister_id: Principal) -> Self {
        Self { canister_id }
    }

    #[inline(always)]
    pub async fn _add_event_listeners(&self, request: AddEventListenersRequest) -> CallResult<()> {
        call(self.canister_id, "_add_event_listeners", (request,)).await
    }

    #[inline(always)]
    pub async fn _remove_event_listeners(
        &self,
        request: RemoveEventListenersRequest,
    ) -> CallResult<(RemoveEventListenersResponse,)> {
        call(self.canister_id, "_remove_event_listeners", (request,)).await
    }

    #[inline(always)]
    pub async fn _become_event_listener(
        &self,
        payload: BecomeEventListenerRequest,
    ) -> CallResult<()> {
        call(self.canister_id, "_become_event_listener", (payload,)).await
    }

    #[inline(always)]
    pub async fn _stop_being_event_listener(
        &self,
        request: StopBeingEventListenerRequest,
    ) -> CallResult<(StopBeingEventListenerResponse,)> {
        call(self.canister_id, "_stop_being_event_listener", (request,)).await
    }

    #[inline(always)]
    pub async fn _get_event_listeners(
        &self,
        request: GetEventListenersRequest,
    ) -> CallResult<(GetEventListenersResponse,)> {
        call(self.canister_id, "_get_event_listeners", (request,)).await
    }
}
