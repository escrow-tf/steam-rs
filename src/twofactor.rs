use reqwest::Method;
use serde::Deserialize;

use crate::{
    steamproto::{CTwoFactorTimeRequest, CTwoFactorTimeResponse},
    transport::PrivateRequest,
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

impl From<QueryTimeRequest> for PrivateRequest<CTwoFactorTimeRequest, CTwoFactorTimeResponse> {
    fn from(_value: QueryTimeRequest) -> Self {
        let request = CTwoFactorTimeRequest::default();

        PrivateRequest::builder()
            .method(Method::POST)
            .path("/ITwoFactorService/QueryTime/v0001".to_string())
            .data(request)
            .build()
    }
}

// TODO: do we need SteamTime() and AlignTime()?
