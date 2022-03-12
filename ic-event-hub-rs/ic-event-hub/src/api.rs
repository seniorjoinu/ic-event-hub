use async_trait::async_trait;
use ic_cdk::api::call::CallResult;
use ic_cdk::call;
use ic_cdk::export::candid::Principal;

use crate::types::{
    GetSubscribersRequest, GetSubscribersResponse, SubscribeRequest, UnsubscribeRequest,
};

#[async_trait]
pub trait IEventHubClient {
    async fn subscribe(&self, payload: SubscribeRequest) -> CallResult<()>;
    async fn unsubscribe(&self, request: UnsubscribeRequest) -> CallResult<()>;
    async fn get_subscribers(
        &self,
        request: GetSubscribersRequest,
    ) -> CallResult<(GetSubscribersResponse,)>;
}

#[async_trait]
impl IEventHubClient for Principal {
    async fn subscribe(&self, req: SubscribeRequest) -> CallResult<()> {
        call(*self, "subscribe", (req,)).await
    }

    async fn unsubscribe(&self, req: UnsubscribeRequest) -> CallResult<()> {
        call(*self, "unsubscribe", (req,)).await
    }

    async fn get_subscribers(
        &self,
        req: GetSubscribersRequest,
    ) -> CallResult<(GetSubscribersResponse,)> {
        call(*self, "get_subscribers", (req,)).await
    }
}
