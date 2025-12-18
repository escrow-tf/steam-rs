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

use std::{collections::HashMap, marker::PhantomData, str::Utf8Error, sync::Arc};

use base64::{Engine, prelude::BASE64_STANDARD};
use bytes::Bytes;
use derive_more::From;
use reqwest::{
    Client, Method, Request, Response, Url, cookie::Jar, header::{ACCEPT, HeaderMap, USER_AGENT}, multipart
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use type_state_builder::TypeStateBuilder;
use url::ParseError;

use crate::steamlang;

const WEB_API_BASE_URL: &str = "https://api.steampowered.com";

#[derive(Clone)]
pub struct PublicTransport {
    client: Client,
    retry_client: ClientWithMiddleware,
    api_key: String,
}

#[derive(Debug, TypeStateBuilder)]
pub struct PublicTransportRequest<'a, R: DeserializeOwned> {
    // TODO: cache_ttl
    #[builder(default = false)]
    can_retry: bool,

    #[builder(default = false)]
    requires_api_key: bool,

    #[builder(default = HashMap::new())]
    params: HashMap<String, String>,

    #[builder(default = WEB_API_BASE_URL)]
    base_url: &'a str,

    #[builder(required)]
    path: &'a str,

    #[builder(default = PhantomData, skip_setter)]
    phantom: PhantomData<R>,
}

#[derive(Error, Debug)]
pub enum NewTransportError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    ParseError(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error(transparent)]
    UrlError(#[from] url::ParseError),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error(transparent)]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),

    #[error(transparent)]
    StatusError(#[from] steamlang::EnsureStatusError),

    #[error(transparent)]
    EResultError(#[from] steamlang::EnsureResultError),

    #[error(transparent)]
    ProstDecodeError(#[from] prost::DecodeError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

impl PublicTransport {
    pub fn new(api_key: &str) -> Result<Self, NewTransportError> {
        let client = Client::builder().build()?;
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(4);
        let retry_client = ClientBuilder::new(client.clone())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            client,
            retry_client,
            api_key: api_key.to_string(),
        })
    }

    /// Sends a public Steam API web request. Only support idempodent GET requests.
    ///
    /// ## Errors
    ///
    /// See [SendError].
    pub async fn send<'a, R: DeserializeOwned>(
        &self,
        request: PublicTransportRequest<'a, R>,
    ) -> Result<R, SendError> {
        let mut url = Url::try_from(request.base_url)?;
        url.set_path(request.path);

        for (param, value) in &request.params {
            url.query_pairs_mut().append_pair(param, value);
        }

        if request.requires_api_key {
            url.query_pairs_mut().append_pair("key", &self.api_key);
        }

        let mut http_request = Request::new(Method::GET, url);
        let headers = http_request.headers_mut();

        headers.insert(ACCEPT, "application/json, text/plain, */*".parse().unwrap());
        headers.insert(USER_AGENT, "okhttp/4.9.2".parse().unwrap());

        let http_response = if request.can_retry {
            self.retry_client.execute(http_request).await?
        } else {
            self.client.execute(http_request).await?
        };

        let http_response = http_response;

        steamlang::ensure_success(&http_response)?;
        steamlang::ensure_eresult(&http_response)?;

        http_response.json().await.map_err(SendError::ReqwestError)
    }
}

#[derive(Debug, Clone)]
pub struct PrivateTransport {
    jar: Arc<Jar>,
    client: ClientWithMiddleware,
    retry_client: ClientWithMiddleware,
}

pub enum TransportBody1 {
    Empty,
    FormValues(HashMap<String, String>),
    MultipartValues(HashMap<String, String>),
    Protobuf(Box<dyn prost::Message>),
}

impl std::fmt::Debug for TransportBody1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::FormValues(arg0) => f.debug_tuple("FormValues").field(arg0).finish(),
            Self::MultipartValues(arg0) => f.debug_tuple("MultipartValues").field(arg0).finish(),
            Self::Protobuf(_) => f.debug_tuple("Protobuf").finish(),
        }
    }
}

pub trait TransportBody {
    fn transform(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> reqwest_middleware::RequestBuilder;
}

#[derive(Default)]
pub struct EmptyBody;

impl TransportBody for EmptyBody {
    fn transform(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> reqwest_middleware::RequestBuilder {
        request
    }
}

#[derive(Debug, derive_more::From)]
pub struct UrlEncodedBody<T: Serialize + Sized>(T);

impl<T: Serialize + Sized> TransportBody for UrlEncodedBody<T> {
    fn transform(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> reqwest_middleware::RequestBuilder {
        request.form(&self.0)
    }
}

#[derive(From)]
pub struct EncodedProtobufBody<T: prost::Message>(T);

impl<T: prost::Message> TransportBody for EncodedProtobufBody<T> {
    fn transform(
        &self,
        request: reqwest_middleware::RequestBuilder,
    ) -> reqwest_middleware::RequestBuilder {
        let bytes = self.0.encode_to_vec();
        let encoded = BASE64_STANDARD.encode(bytes);

        let form = multipart::Form::new().text("input_protobuf_encoded", encoded);

        request.multipart(form)
    }
}

#[derive(Debug, TypeStateBuilder)]
pub struct PrivateTransportRequest<'a, B: TransportBody, R> {
    // TODO: cache_ttl
    #[builder(default = false)]
    can_retry: bool,

    #[builder(default = WEB_API_BASE_URL)]
    base_url: &'a str,

    #[builder(required)]
    path: &'a str,

    #[builder(default = HashMap::new())]
    params: HashMap<String, String>,

    #[builder(default = Method::POST)]
    method: Method,

    #[builder(required)]
    body: B,

    // TODO: do I need a Content Type?
    #[builder(default = HeaderMap::default())]
    headers: HeaderMap,

    #[builder(default = PhantomData, skip_setter)]
    phantom: PhantomData<R>,
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

    pub async fn send<'a, B: TransportBody, R>(
        &self,
        request: PrivateTransportRequest<'a, B, R>,
    ) -> Result<Response, SendError> {
        let mut url = Url::try_from(request.base_url)?;
        url.set_path(request.path);

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

        let mut http_request = http_client
            .request(request.method, url)
            .headers(headers);

        http_request = request.body.transform(http_request);

        let response = http_request.send().await?;

        Ok(response)
    }

    pub async fn send_json<'a, B: TransportBody, R: for<'de> Deserialize<'de>>(
        &self,
        request: PrivateTransportRequest<'a, B, R>,
    ) -> Result<R, SendError> {
        let response = self.send(request).await?;
        let result: R = response.json().await?;
        Ok(result)
    }

    pub async fn send_proto<'a, B: TransportBody, R: prost::Message + Default>(
        &self,
        request: PrivateTransportRequest<'a, B, R>,
    ) -> Result<R, SendError> {
        let response = self.send(request).await?;
        // TODO: check if steam actually returns raw bytes instead of base64-encoded.
        let response = response.bytes().await?;
        R::decode(response).map_err(SendError::ProstDecodeError)
    }
}
