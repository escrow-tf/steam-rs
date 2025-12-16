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
    status: i32,
    status_detail: String,
    num_backpack_slots: i32,
    // TODO: items,
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
mod tests {
    use crate::{
        tf2econ::PlayerItemsRequest,
        transport::{PublicTransport, PublicTransportRequest},
    };

    async fn do_transport_request() {
        let request = PlayerItemsRequest {
            steam_id: "76561197960287930".parse().unwrap(),
        };
        let request: PublicTransportRequest<_> = request.into();

        let transport = PublicTransport::new("").unwrap();
        _ = transport.send(request).await.unwrap();
    }
}
