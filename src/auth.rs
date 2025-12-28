use anyhow::anyhow;
use prost::Message;
use rsa::{BigUint, RsaPublicKey};
use serde::Deserialize;
use steam_encode::Decode;
use type_state_builder::TypeStateBuilder;

use crate::{steamid::SteamID, steamproto::{CAuthenticationGetPasswordRsaPublicKeyRequest, CAuthenticationGetPasswordRsaPublicKeyResponse}, transport::{PrivateRequest, WEB_API_BASE_URL}};

#[derive(Debug, Deserialize)]
pub struct Claims {
    #[serde(rename = "sub")]
    pub subject: SteamID,

    #[serde(rename = "iat")]
    pub issued_at: u64,

    #[serde(rename = "exp")]
    pub expires_at: u64,
}

#[derive(Debug, TypeStateBuilder)]
pub struct RsaKeyRequest {
    #[builder(required)]
    pub account_name: String,
}

#[derive(Debug)]
pub struct RsaKeyResponse {
    pub public_key: RsaPublicKey,
    pub timestamp: u64,
}

impl Decode for RsaKeyResponse {
    async fn decode(response: reqwest::Response) -> anyhow::Result<Self> {
        let bytes = response.bytes().await?;
        let result: CAuthenticationGetPasswordRsaPublicKeyResponse = Message::decode(bytes)?;

        let key_modulus = result.publickey_mod()
            .parse::<BigUint>()
            .map_err(|err| anyhow!("error parsing the public key's modulus: {err}"))?;

        let key_exponent = result.publickey_exp()
            .parse::<BigUint>()
            .map_err(|err| anyhow!("error parsing the public key's exponent: {err}"))?;

        Ok(Self {
            public_key: RsaPublicKey::new(key_modulus, key_exponent)?,
            timestamp: result.timestamp(),
        })
    }
}

impl From<RsaKeyRequest> for PrivateRequest<CAuthenticationGetPasswordRsaPublicKeyRequest, RsaKeyResponse> {
    fn from(request: RsaKeyRequest) -> Self {
        let request = CAuthenticationGetPasswordRsaPublicKeyRequest {
            account_name: Some(request.account_name),
        };

        PrivateRequest::builder()
            .base_url(WEB_API_BASE_URL)
            .path("/IAuthenticationService/GetPasswordRSAPublicKey/v1/")
            .data(request)
            .build()
    }
}