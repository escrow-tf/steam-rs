use derive_more::From;
use serde::{Deserialize, Serialize};
use transport_derive::{Decode, Encode};
use type_state_builder::TypeStateBuilder;

use crate::{steamid::SteamID, transport::PublicTransportRequest};

#[derive(Debug, TypeStateBuilder, Serialize, Encode)]
#[encode(query)]
pub struct PlayerItemsRequest {
    #[builder(required)]
    #[serde(rename = "steamid")]
    steam_id: SteamID,
}

#[derive(Debug, Deserialize)]
pub struct PlayerItemsResult {
    pub status: i32,
    pub status_detail: Option<String>,
    pub num_backpack_slots: Option<i32>,
    pub items: Option<Vec<Item>>,
    // TODO: items,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: i32,
    pub original_id: i32,
    pub def_index: i32,
    pub level: i32,
    pub quality: i32,
    pub inventory: i64,
    pub quantity: i32,
    pub origin: i32,

    #[serde(default)]
    pub cannot_trade: bool,

    #[serde(default)]
    pub style: i32,

    #[serde(default)]
    pub cannot_craft: bool,

    pub custom_name: Option<String>,

    #[serde(rename = "custom_desc")]
    pub custom_description: Option<String>,
    pub attributes: Option<Vec<Attribute>>,
    pub equipped: Option<Vec<EquipInfo>>,
}

#[derive(Debug, Deserialize, From)]
pub enum AttributeValue {
    Int(i64),
    String(String),
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    #[serde(rename = "defindex")]
    pub def_index: i32,
    pub value: AttributeValue,
    pub float_value: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct EquipInfo {
    pub class: i32,
    pub slot: i32,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct PlayerItemsResponse {
    pub result: PlayerItemsResult,
}

impl From<PlayerItemsRequest> for PublicTransportRequest<PlayerItemsRequest, PlayerItemsResponse> {
    fn from(request: PlayerItemsRequest) -> Self {
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .path("/IEconItems_440/GetPlayerItems/v1/")
            .data(request)
            .build()
    }
}

#[cfg(test)]
mod tests {}
