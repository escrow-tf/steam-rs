use reqwest::Method;
use serde::Deserialize;

use crate::{
    steamproto::{CTwoFactorTimeRequest, CTwoFactorTimeResponse},
    transport::{EncodedProtobufBody, PrivateTransportRequest},
};

#[derive(Debug, Default)]
pub struct QueryTimeRequest;

impl QueryTimeRequest {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryTimeResponse {}

impl<'a> From<QueryTimeRequest>
    for PrivateTransportRequest<'a, EncodedProtobufBody<CTwoFactorTimeRequest>, CTwoFactorTimeResponse>
{
    fn from(_value: QueryTimeRequest) -> Self {
        let request = CTwoFactorTimeRequest::default();

        PrivateTransportRequest::builder()
            .method(Method::POST)
            .path("/ITwoFactorService/QueryTime/v0001".to_string())
            .body(request.into())
            .build()
    }
}

// TODO: do we need SteamTime() and AlignTime()?
