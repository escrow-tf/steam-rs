use derive_more::From;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use transport_derive::{Decode, Encode};

use crate::transport::{COMMUNITY_BASE_URL, PrivateTransportRequest};

#[derive(Debug, From, Serialize, Deserialize)]
pub struct OfferId(u64);

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct ActionResponse {
    #[serde(rename = "tradeofferid")]
    pub trade_offer_id: OfferId,
}

#[derive(Debug, Serialize, Encode)]
#[encode(form)]
pub struct AcceptOfferRequest {
    pub id: OfferId,
    pub session_id: String,
}

impl From<AcceptOfferRequest> for PrivateTransportRequest<AcceptOfferRequest, ActionResponse> {
    fn from(request: AcceptOfferRequest) -> Self {
        PrivateTransportRequest::builder()
            .method(Method::POST)
            .base_url(COMMUNITY_BASE_URL)
            .path(format!("/tradeoffer/{}/accept", request.id.0))
            .data(request)
            .build()
    }
}

#[derive(Debug, Serialize, Encode)]
#[encode(form)]
pub struct DeclineOfferRequest {
    pub id: OfferId,
}

impl From<DeclineOfferRequest> for PrivateTransportRequest<DeclineOfferRequest, ActionResponse> {
    fn from(request: DeclineOfferRequest) -> Self {
        PrivateTransportRequest::builder()
            .method(Method::POST)
            .base_url(COMMUNITY_BASE_URL)
            .path(format!("/tradeoffer/{}/decline", request.id.0))
            .data(request)
            .build()
    }
}

#[derive(Debug, Serialize, Encode)]
#[encode(form)]
pub struct CancelOfferRequest {
    pub id: OfferId,
}

impl From<CancelOfferRequest> for PrivateTransportRequest<CancelOfferRequest, ActionResponse> {
    fn from(request: CancelOfferRequest) -> Self {
        PrivateTransportRequest::builder()
            .method(Method::POST)
            .base_url(COMMUNITY_BASE_URL)
            .path(format!("/tradeoffer/{}/cancel", request.id.0))
            .data(request)
            .build()
    }
}
