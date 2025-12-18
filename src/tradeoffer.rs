use derive_more::From;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::transport::{PrivateTransportRequest, UrlEncodedBody};

const COMMUNITY_BASE_URL: &str = "https://steamcommunity.com";

#[derive(Debug, From, Serialize, Deserialize)]
pub struct OfferId(u64);

#[derive(Debug, Serialize)]
pub struct AcceptOfferRequest {
    pub id: OfferId,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ActionResponse {
    #[serde(rename = "tradeofferid")]
    pub trade_offer_id: OfferId,
}

impl<'a> From<AcceptOfferRequest>
    for PrivateTransportRequest<'a, UrlEncodedBody<AcceptOfferRequest>, ActionResponse>
{
    fn from(request: AcceptOfferRequest) -> Self {
        PrivateTransportRequest::builder()
            .method(Method::POST)
            .base_url(COMMUNITY_BASE_URL)
            .path("/ITwoFactorService/QueryTime/v0001")
            .body(request.into())
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
