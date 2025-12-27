/*
the transport handles requests over HTTP. There are three kinds of transports:

- PublicTransport
    - a transport intended to be used to interact with the public Web APIs.
    - most public APIs require a Web API key
    - does not save cookies
    - does not send credential cookies
    - requests are almost always query params
    - responses are almost always JSON

- PrivateTransport
    - interacts with the non-public APIs
    - saves cookies set by responses
    - sends credential cookies
    - most stateless requests (GET, HEAD, OPTION) use query params
    - most stateful request (POST, PUT, PATCH, DELETE) use protobuf bodies
    - most responses use protobuf bodies
*/

use std::{
    collections::HashMap,
    marker::PhantomData,
    str::{FromStr, Utf8Error},
    sync::Arc,
};

use reqwest::{
    Client, Method, Url,
    cookie::Jar,
    header::{ACCEPT, HeaderMap, USER_AGENT},
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use thiserror::Error;
use transport::{Decode, Encode};
use type_state_builder::TypeStateBuilder;

use crate::steamlang;

pub const WEB_API_BASE_URL: &str = "https://api.steampowered.com";
pub const COMMUNITY_BASE_URL: &str = "https://www.steamcommunity.com";

#[derive(Clone)]
/// Handles requests to Steam's Public Web API.
pub struct PublicTransport {
    client: ClientWithMiddleware,
    retry_client: ClientWithMiddleware,
    api_key: String,
}

#[derive(Debug, TypeStateBuilder)]
#[builder(impl_into)]
pub struct PublicRequest<I: Encode, O: Decode> {
    // TODO: cache_ttl
    #[builder(default = false)]
    can_retry: bool,

    #[builder(default = false)]
    requires_api_key: bool,

    #[builder(default = Url::from_str(WEB_API_BASE_URL).unwrap(), converter = |url: &str| Url::from_str(url).unwrap())]
    base_url: Url,

    #[builder(required)]
    path: String,

    #[builder(required)]
    data: I,

    #[builder(default = PhantomData, skip_setter)]
    out_phantom: PhantomData<O>,
}

#[derive(Error, Debug)]
pub enum NewTransportError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Reqwest(#[from] reqwest_middleware::Error),

    #[error(transparent)]
    Status(#[from] steamlang::EnsureStatusError),

    #[error(transparent)]
    SteamEResult(#[from] steamlang::EnsureResultError),

    #[error(transparent)]
    Prost(#[from] prost::DecodeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error(transparent)]
    Decode(#[from] anyhow::Error),
}

impl PublicTransport {
    /// Create a new [`PublicTransport`], used for handling requests to Steam's Public Web API.
    ///
    /// ### Errors
    ///
    /// Only fails if [`reqwest::ClientBuilder`] fails to initialize a TLS backend.
    pub fn new(api_key: &str) -> Result<Self, NewTransportError> {
        let client = Client::builder().build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(4);
        let retry_client = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            client: ClientWithMiddleware::from(client),
            retry_client,
            api_key: api_key.to_string(),
        })
    }

    /// Sends a public Steam API web request. Only support idempodent GET requests.
    ///
    /// ## Errors
    ///
    /// See [SendError].
    pub async fn send<I: Encode, O: Decode>(&self, request: PublicRequest<I, O>) -> Result<O, SendError> {
        let mut url = request.base_url.clone();
        url.set_path(request.path.as_str());

        // for (param, value) in &request.params {
        //     url.query_pairs_mut().append_pair(param, value);
        // }

        if request.requires_api_key {
            url.query_pairs_mut().append_pair("key", &self.api_key);
        }

        let http_client = if request.can_retry {
            self.retry_client.clone()
        } else {
            self.client.clone()
        };

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json, text/plain, */*".parse().unwrap());
        headers.insert(USER_AGENT, "okhttp/4.9.2".parse().unwrap());

        let http_request = http_client.request(Method::GET, url).headers(headers);

        let http_request = request.data.encode(http_request);
        let http_response = http_request.send().await?;

        steamlang::ensure_success(&http_response)?;
        steamlang::ensure_eresult(&http_response)?;

        O::decode(http_response).await.map_err(SendError::Decode)
    }
}

#[derive(Debug, Clone)]
pub struct PrivateTransport {
    jar: Arc<Jar>,
    client: ClientWithMiddleware,
    retry_client: ClientWithMiddleware,
}

#[derive(Debug, TypeStateBuilder)]
#[builder(impl_into)]
pub struct PrivateRequest<I: Encode, O: Decode> {
    // TODO: cache_ttl
    #[builder(default = false)]
    can_retry: bool,

    #[builder(default = Url::from_str(COMMUNITY_BASE_URL).unwrap(), converter = |url: &str| Url::from_str(url).unwrap())]
    base_url: Url,

    #[builder(required)]
    path: String,

    #[builder(default = HashMap::new())]
    params: HashMap<String, String>,

    // TODO: do I need a Content Type?
    #[builder(default = HeaderMap::default())]
    headers: HeaderMap,

    #[builder(default = Method::POST)]
    method: Method,

    #[builder(required)]
    data: I,

    #[builder(default = PhantomData, skip_setter)]
    out_phantom: PhantomData<O>,
}

impl PrivateTransport {
    pub fn new() -> Result<Self, NewTransportError> {
        // TODO: support Steam Client requests, not just mobile/web
        let jar = Jar::default();
        let cookie_url = "steamcommunity.com".parse::<Url>()?;
        jar.add_cookie_str("mobileClient=android", &cookie_url);
        jar.add_cookie_str("mobileClientVersion=777777 3.10.3", &cookie_url);

        let jar = Arc::new(jar);

        let client = Client::builder().cookie_provider(jar.clone()).build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(4);
        let retry_client = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            jar,
            client: ClientWithMiddleware::from(client),
            retry_client,
        })
    }

    pub async fn send<I: Encode, O: Decode>(&self, request: PrivateRequest<I, O>) -> Result<O, SendError> {
        let mut url = request.base_url.clone();
        url.set_path(&request.path);

        for (param, value) in &request.params {
            url.query_pairs_mut().append_pair(param, value);
        }

        let http_client = if request.can_retry {
            self.retry_client.clone()
        } else {
            self.client.clone()
        };

        let mut headers = request.headers.clone();
        headers.insert(ACCEPT, "application/json, text/plain, */*".parse().unwrap());
        headers.insert(USER_AGENT, "okhttp/4.9.2".parse().unwrap());

        let http_request = http_client.request(request.method, url).headers(headers);
        let http_request = request.data.encode(http_request);
        let http_response = http_request.send().await?;

        steamlang::ensure_success(&http_response)?;
        steamlang::ensure_eresult(&http_response)?;

        O::decode(http_response).await.map_err(SendError::Decode)
    }
}
