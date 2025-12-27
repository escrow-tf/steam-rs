use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use transport_derive::{Decode, Encode};
use type_state_builder::TypeStateBuilder;

use crate::{steamid::SteamID, totp::ConfirmationKey, transport::PrivateRequest};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ConfirmationId(String);

#[derive(Debug, Deserialize)]
pub enum ConfirmationType {
    Invalid = 0,
    Trade = 1,
    MarketListing = 2,
    Other = 3,
}

#[derive(Debug, Deserialize)]
pub struct Confirmation {
    pub id: ConfirmationId,
    pub r#type: ConfirmationType,
    pub creator_id: String,
    pub none: String,
    pub type_name: String,
    pub headline: String,
    pub summary: Vec<String>,
    pub creation_time: i64,
    pub icon: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct ListResponse {
    pub success: bool,

    #[serde(default, rename = "needsauth")]
    pub needs_auth: bool,

    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub details: String,

    #[serde(rename = "conf")]
    pub confirmations: Vec<Confirmation>,
}

#[derive(Debug, TypeStateBuilder, Encode)]
#[encode(empty)]
pub struct ListRequest {
    #[builder(required)]
    steam_id: SteamID,

    #[builder(required)]
    device_id: String,

    #[builder(required)]
    confirmation_key: ConfirmationKey,
}

impl From<ListRequest> for PrivateRequest<ListRequest, ListResponse> {
    fn from(request: ListRequest) -> Self {
        let params = HashMap::from([
            ("p".to_string(), request.device_id.clone()),
            ("a".to_string(), request.steam_id.to_string()),
            ("k".to_string(), request.confirmation_key.to_string()),
            ("t".to_string(), request.confirmation_key.unix_time().to_string()),
            ("m".to_string(), "react".to_string()),
            ("tag".to_string(), request.confirmation_key.tag().to_string()),
        ]);

        PrivateRequest::builder()
            .path("/mobileconf/getlist")
            .data(request)
            .params(params)
            .method(Method::GET)
            .build()
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmationTradeOffer {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct DetailsPageResponse {
    #[serde(rename = "tradeoffer")]
    pub trade_offer: Option<ConfirmationTradeOffer>,
}

#[derive(Debug, TypeStateBuilder, Encode)]
#[encode(empty)]
pub struct DetailsPageRequest {
    #[builder(required)]
    steam_id: SteamID,

    #[builder(required)]
    device_id: String,

    #[builder(required)]
    confirmation_key: ConfirmationKey,

    #[builder(required)]
    id: ConfirmationId,
}

impl From<DetailsPageRequest> for PrivateRequest<DetailsPageRequest, DetailsPageResponse> {
    fn from(request: DetailsPageRequest) -> Self {
        let params = HashMap::from([
            ("p".to_string(), request.device_id.clone()),
            ("a".to_string(), request.steam_id.to_string()),
            ("k".to_string(), request.confirmation_key.to_string()),
            ("t".to_string(), request.confirmation_key.unix_time().to_string()),
            ("m".to_string(), "react".to_string()),
            ("tag".to_string(), request.confirmation_key.tag().to_string()),
        ]);

        PrivateRequest::builder()
            .path(format!("/mobileconf/detailspage/{}", request.id.0))
            .data(request)
            .params(params)
            .method(Method::GET)
            .build()
    }
}

#[derive(Debug, Serialize, Encode)]
#[encode(form)]
pub struct AjaxRequest {
    #[serde(rename = "p")]
    device_id: String,

    #[serde(rename = "a")]
    steam_id: SteamID,

    #[serde(rename = "k")]
    key: String,

    #[serde(rename = "t")]
    time: u64,

    #[serde(rename = "tag")]
    tag: String,

    #[serde(default = "react", rename = "m")]
    react: String,

    #[serde(rename = "op")]
    operation: String,

    #[serde(rename = "cid")]
    confirmation_id: ConfirmationId,

    #[serde(rename = "ck")]
    confirmation_nonce: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct AcceptResponse {
    pub success: bool,

    #[serde(default, rename = "needsauth")]
    pub needs_auth: bool,

    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub details: String,
}

#[derive(Debug, TypeStateBuilder)]
pub struct AcceptRequest {
    #[builder(required)]
    steam_id: SteamID,

    #[builder(required)]
    device_id: String,

    #[builder(required)]
    confirmation_key: ConfirmationKey,

    #[builder(required)]
    id: ConfirmationId,

    #[builder(required)]
    nonce: String,
}

impl From<AcceptRequest> for PrivateRequest<AjaxRequest, AcceptResponse> {
    fn from(request: AcceptRequest) -> Self {
        let request = AjaxRequest {
            device_id: request.device_id,
            steam_id: request.steam_id,
            key: request.confirmation_key.to_string(),
            time: request.confirmation_key.unix_time(),
            tag: request.confirmation_key.tag_owned(),
            react: "react".to_string(),
            operation: "allow".to_string(),
            confirmation_id: request.id,
            confirmation_nonce: request.nonce,
        };

        PrivateRequest::builder()
            .path("/mobileconf/ajaxop")
            .data(request)
            .build()
    }
}

#[derive(Debug, TypeStateBuilder)]
pub struct DeclineRequest {
    #[builder(required)]
    steam_id: SteamID,

    #[builder(required)]
    device_id: String,

    #[builder(required)]
    confirmation_key: ConfirmationKey,

    #[builder(required)]
    id: ConfirmationId,

    #[builder(required)]
    nonce: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct DeclineResponse {
    pub success: bool,

    #[serde(default, rename = "needsauth")]
    pub needs_auth: bool,

    #[serde(default)]
    pub message: String,

    #[serde(default)]
    pub details: String,
}

impl From<DeclineRequest> for PrivateRequest<AjaxRequest, AcceptResponse> {
    fn from(request: DeclineRequest) -> Self {
        let request = AjaxRequest {
            device_id: request.device_id,
            steam_id: request.steam_id,
            key: request.confirmation_key.to_string(),
            time: request.confirmation_key.unix_time(),
            tag: request.confirmation_key.tag_owned(),
            react: "react".to_string(),
            operation: "cancel".to_string(),
            confirmation_id: request.id,
            confirmation_nonce: request.nonce,
        };

        PrivateRequest::builder()
            .path("/mobileconf/ajaxop")
            .data(request)
            .build()
    }
}
