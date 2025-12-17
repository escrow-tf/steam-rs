use derive_more::From;
use serde::Deserialize;
use type_state_builder::TypeStateBuilder;

use crate::{steamid::SteamID, transport::PublicTransportRequest};

const WEB_API_BASE_URL: &str = "https://api.steampowered.com";

#[derive(Debug, TypeStateBuilder)]
pub struct PlayerItemsRequest {
    #[builder(required)]
    steam_id: SteamID,
}

#[derive(Debug, Deserialize)]
pub struct PlayerItemsResult {
    pub status: i32,
    pub status_detail: String,
    pub num_backpack_slots: i32,
    pub items: Vec<Item>,
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
    pub cannot_trade: bool,
    pub style: i32,
    pub cannot_craft: bool,
    pub custom_name: Option<String>,

    #[serde(rename = "custom_desc")]
    pub custom_description: Option<String>,
    pub attributes: Vec<Attribute>,
    pub equipped: Vec<EquipInfo>,
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

#[derive(Debug, Deserialize)]
pub struct PlayerItemsResponse {
    pub result: PlayerItemsResult,
}

impl<'a> From<PlayerItemsRequest> for PublicTransportRequest<'a, PlayerItemsResponse> {
    fn from(request: PlayerItemsRequest) -> Self {
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .params(vec![("steamid".to_string(), request.steam_id.to_string())])
            .base_url(WEB_API_BASE_URL)
            .path("/IEconItems_440/GetPlayerItems/v1/")
            .build()
    }
}

#[cfg(test)]
mod tests {}
