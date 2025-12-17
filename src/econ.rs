use std::collections::HashMap;

use crate::{community, steamid::SteamID};
use serde::Deserialize;
use type_state_builder::TypeStateBuilder;

use crate::transport::PublicTransportRequest;

#[derive(Debug, Deserialize)]
pub enum TradeOfferState {
    /// Invalid - Invalid
    Invalid = 1,
    /// Active - This trade offer has been sent, neither party has acted on it yet.
    Active = 2,
    /// Accepted - The trade offer was accepted by the recipient and items were exchanged.
    Accepted = 3,
    /// Countered - The recipient made a counter-offer
    Countered = 4,
    /// Expired - The trade offer was not accepted before the expiration date
    Expired = 5,
    /// Canceled - The sender cancelled the offer
    Canceled = 6,
    /// Declined - The recipient declined the offer
    Declined = 7,
    /// InvalidItems - Some of the items in the offer are no longer available (indicated by the
    /// missing flag in the output)
    InvalidItems = 8,
    /// CreatedNeedsConfirmation - The offer hasn't been sent yet and is awaiting email/mobile
    /// confirmation. The offer is only visible to the sender.
    CreatedNeedsConfirmation = 9,
    /// CanceledBySecondFactor - Either party canceled the offer via email/mobile. The offer is
    /// visible to both parties, even if the sender canceled it before it was sent.
    CanceledBySecondFactor = 10,
    /// InEscrow - The trade has been placed on hold. The items involved in the trade have all
    /// been removed from both parties' inventories and will be automatically delivered in the future.
    InEscrow = 11,
}

#[derive(Debug, Deserialize)]
pub enum OfferConfirmationMethod {
    InvalidOffer = 0,
    EmailOffer = 1,
    MobileAppOffer = 2,
}

#[derive(Debug, Deserialize)]
pub struct TradeOffer {
    pub trade_offer_id: u64,
    pub trade_id: u64,

    #[serde(rename = "accountid_other")]
    pub other_account_id: u64,
    pub other_steam_id: SteamID,
    pub message: String,
    pub expiration_time: u32,

    #[serde(rename = "trade_offer_state")]
    pub state: TradeOfferState,

    #[serde(rename = "items_to_give")]
    pub to_give: Vec<community::Asset>,

    #[serde(rename = "items_to_receive")]
    pub to_receive: Vec<community::Asset>,

    pub is_our_offer: bool,
    pub time_created: u32,
    pub time_updated: u32,
    pub escrow_end_date: u32,
    pub confirmation_method: OfferConfirmationMethod,
}

#[derive(Debug, TypeStateBuilder)]
pub struct TradeOfferRequest<'a> {
    #[builder(required)]
    trade_id: u64,

    #[builder(default = "en_us")]
    language: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct TradeOfferResponse {
    pub offer: TradeOffer,
    pub descriptions: Vec<community::Description>,
}

impl<'a> From<TradeOfferRequest<'a>> for PublicTransportRequest<'a, TradeOfferResponse> {
    fn from(request: TradeOfferRequest) -> Self {
        let params = HashMap::from([
            ("tradeofferid".to_string(), request.trade_id.to_string()),
            ("language".to_string(), request.language.to_string()),
        ]);

        // TODO: do we even need the api key?
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .params(params)
            .path("/IEconService/GetTradeOffer/v1/")
            .build()
    }
}

#[derive(Debug)]
pub enum OffersFilter {
    None,
    OnlyActive,
    OnlyHistorical,
    OnlyHistoricalWithCutoff(u32),
}

#[derive(Debug, TypeStateBuilder)]
pub struct TradeOffersRequest<'a> {
    #[builder(required)]
    get_sent: bool,

    #[builder(required)]
    get_received: bool,

    #[builder(required)]
    get_descriptions: bool,

    #[builder(default = OffersFilter::None)]
    filter: OffersFilter,

    #[builder(default = "en_us")]
    language: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct TradeOffersResponse {
    pub sent: Vec<TradeOffer>,
    pub received: Vec<TradeOffer>,
    pub descriptions: Vec<community::Description>,
}

impl<'a> From<TradeOffersRequest<'a>> for PublicTransportRequest<'a, TradeOffersResponse> {
    fn from(request: TradeOffersRequest) -> Self {
        let mut params = HashMap::from([(String::from("language"), request.language.to_string())]);

        match request.filter {
            OffersFilter::None => {}
            OffersFilter::OnlyActive => {
                params.insert(String::from("active_only"), String::from("1"));
            }
            OffersFilter::OnlyHistorical => {
                params.insert(String::from("historical_only"), String::from("1"));
            }
            OffersFilter::OnlyHistoricalWithCutoff(cutoff) => {
                params.insert(String::from("historical_only"), String::from("1"));
                params.insert(String::from("time_historical_cutoff"), cutoff.to_string());
            }
        };

        if request.get_sent {
            params.insert(String::from("get_sent_offers"), String::from("1"));
        }

        if request.get_received {
            params.insert(String::from("get_received_offers"), String::from("1"));
        }

        if request.get_descriptions {
            params.insert(String::from("get_descriptions"), String::from("1"));
        }

        // TODO: do we even need the api key?
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .params(params)
            .path("/IEconService/GetTradeOffer/v1/")
            .build()
    }
}

#[cfg(test)]
mod tests {}
