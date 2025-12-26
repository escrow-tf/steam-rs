use std::collections::HashMap;

use crate::{community, steamid::SteamID};
use serde::{Deserialize, Serialize};
use transport::Encode;
use transport_derive::{Decode, Encode};
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

#[derive(Debug, TypeStateBuilder, Serialize, Encode)]
#[encode(query)]
pub struct TradeOfferRequest {
    #[builder(required)]
    #[serde(rename = "tradeofferid")]
    trade_id: u64,

    #[builder(default = String::from("en_us"))]
    language: String,
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct TradeOfferResponse {
    pub offer: TradeOffer,
    pub descriptions: Vec<community::Description>,
}

impl From<TradeOfferRequest> for PublicTransportRequest<TradeOfferRequest, TradeOfferResponse> {
    fn from(request: TradeOfferRequest) -> Self {
        // TODO: do we even need the api key?
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .path("/IEconService/GetTradeOffer/v1/")
            .data(request)
            .build()
    }
}

#[derive(Debug)]
pub enum OffersFilter {
    None,
    OnlyActive,
    OnlyHistorical,
    OnlyHistoricalWithCutoff(&'static str),
}

#[derive(Debug, TypeStateBuilder)]
pub struct TradeOffersRequest {
    #[builder(required)]
    get_sent: bool,

    #[builder(required)]
    get_received: bool,

    #[builder(required)]
    get_descriptions: bool,

    #[builder(default = OffersFilter::None)]
    filter: OffersFilter,

    #[builder(default = "en_us")]
    language: &'static str,
}

impl Encode for TradeOffersRequest {
    fn encode(&self, request: reqwest_middleware::RequestBuilder) -> reqwest_middleware::RequestBuilder {
        let mut params = HashMap::from([("language", self.language)]);

        match self.filter {
            OffersFilter::None => {}
            OffersFilter::OnlyActive => {
                params.insert("active_only", "1");
            }
            OffersFilter::OnlyHistorical => {
                params.insert("historical_only", "1");
            }
            OffersFilter::OnlyHistoricalWithCutoff(cutoff) => {
                params.insert("historical_only", "1");
                params.insert("time_historical_cutoff", cutoff);
            }
        };

        if self.get_sent {
            params.insert("get_sent_offers", "1");
        }

        if self.get_received {
            params.insert("get_received_offers", "1");
        }

        if self.get_descriptions {
            params.insert("get_descriptions", "1");
        }

        request.query(&params)
    }
}

#[derive(Debug, Deserialize, Decode)]
#[decode(json)]
pub struct TradeOffersResponse {
    pub sent: Vec<TradeOffer>,
    pub received: Vec<TradeOffer>,
    pub descriptions: Vec<community::Description>,
}

impl From<TradeOffersRequest> for PublicTransportRequest<TradeOffersRequest, TradeOffersResponse> {
    fn from(request: TradeOffersRequest) -> Self {
        // TODO: do we even need the api key?
        PublicTransportRequest::builder()
            .can_retry(true)
            .requires_api_key(true)
            .path("/IEconService/GetTradeOffer/v1/")
            .data(request)
            .build()
    }
}

#[cfg(test)]
mod tests {}
