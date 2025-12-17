use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

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
