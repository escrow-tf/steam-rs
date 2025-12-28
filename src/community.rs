use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use steam_encode_derive::{Decode, Encode};
use type_state_builder::TypeStateBuilder;

use crate::{steamid::SteamID, transport::PrivateRequest};

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Asset {
    pub app_id: u32,

    #[serde_as(as = "DisplayFromStr")]
    pub context_id: u32,

    #[serde_as(as = "DisplayFromStr")]
    pub asset_id: u32,

    #[serde_as(as = "DisplayFromStr")]
    pub class_id: u32,

    #[serde_as(as = "DisplayFromStr")]
    pub instance_id: u32,

    #[serde_as(as = "DisplayFromStr")]
    pub amount: u32,
}

#[derive(Debug, Deserialize)]
pub struct Description {
    pub app_id: i32,
    pub class_id: String,
    #[serde(rename = "instanceid")]
    pub instance_id: String,
    pub currency: i32,
    pub background_color: String,
    pub icon_url: String,
    pub icon_url_large: String,
    pub tradable: i32,
    pub name: String,
    pub name_color: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub market_name: String,
    pub market_hash_name: String,
    pub commodity: i32,
    pub market_tradable_restriction: String,
    pub market_marketable_restriction: String,
    pub marketable: String,
    #[serde(rename = "fraudwarnings")]
    pub fraud_warnings: Option<Vec<String>>,
    pub tags: Vec<Tag>,
    pub lines: Option<Vec<Line>>,
    pub actions: Option<Vec<Action>>,
    pub market_actions: Option<Vec<Action>>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    pub category: String,
    pub internal_name: String,
    pub localized_category_name: String,
    pub localized_tag_name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Line {
    pub value: String,
    pub color: Option<String>,
    #[serde(rename = "type")]
    pub line_type: Option<String>,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub link: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct PlayerInventory {
    pub assets: Vec<Asset>,
    pub descriptions: Vec<Description>,
    pub more_items: Option<u64>,

    #[serde(rename = "last_assetid")]
    pub last_asset_id: Option<u64>,
    pub total_inventory_count: u64,
    pub success: u8,
    pub rwgrsn: u8,
}

#[derive(Debug, TypeStateBuilder, Serialize, Encode)]
#[encode(form)]
pub struct PlayerInventoryRequest {
    #[builder(default = 0)]
    start: u64,

    #[builder(default = 100)]
    count: u64,

    #[builder(default = String::from("en_us"))]
    #[serde(rename = "l")]
    language: String,

    #[builder(required)]
    #[serde(skip_serializing)]
    steam_id: SteamID,

    #[builder(required)]
    #[serde(skip_serializing)]
    app_id: u64,

    #[builder(default = 2)]
    #[serde(skip_serializing)]
    context_id: u64,
}

impl From<PlayerInventoryRequest> for PrivateRequest<PlayerInventoryRequest, PlayerInventory> {
    fn from(request: PlayerInventoryRequest) -> Self {
        let steam_id = request.steam_id.to_string();
        let app_id = request.app_id.to_string();
        let context_id = request.context_id.to_string();

        let steam_id = urlencoding::encode(&steam_id);
        let app_id = urlencoding::encode(&app_id);
        let context_id = urlencoding::encode(&context_id);

        PrivateRequest::builder()
            .method(Method::GET)
            .path(format!("/inventory/{app_id}/{steam_id}/{context_id}"))
            .can_retry(true)
            .data(request)
            .build()
    }
}
