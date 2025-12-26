use derive_more::From;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use transport_derive::{Decode, Encode};

use crate::transport::PrivateTransportRequest;

const COMMUNITY_BASE_URL: &str = "https://steamcommunity.com";

#[derive(Debug, From, Serialize, Deserialize)]
pub struct OfferId(u64);

#[derive(Debug, Serialize, Encode)]
#[encode(form)]
pub struct AcceptOfferRequest {
    pub id: OfferId,
    pub session_id: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct ActionResponse {
    #[serde(rename = "tradeofferid")]
    pub trade_offer_id: OfferId,
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

#[derive(Debug)]
pub struct DeclineOfferRequest {
    pub id: OfferId,
}

#[derive(Debug)]
pub struct CancelOfferRequest {
    pub id: OfferId,
}
