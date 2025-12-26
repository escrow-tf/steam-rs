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
use derive_more::From;
use reqwest::{
    Client, Method, Response, Url,
    cookie::Jar,
    header::{ACCEPT, HeaderMap, USER_AGENT},
    multipart,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use transport::{Decode, Encode};
use type_state_builder::TypeStateBuilder;
use url::ParseError;

use crate::steamlang;

const WEB_API_BASE_URL: &str = "https://api.steampowered.com";

#[derive(Clone)]
pub struct PublicTransport {
    client: ClientWithMiddleware,
    retry_client: ClientWithMiddleware,
    api_key: String,
}

#[derive(Debug, TypeStateBuilder)]
#[builder(impl_into)]
pub struct PublicTransportRequest<I: Encode, O: Decode> {
    // TODO: cache_ttl
    #[builder(default = false)]
    can_retry: bool,

    #[builder(default = false)]
    requires_api_key: bool,

    #[builder(default = String::from(WEB_API_BASE_URL))]
    base_url: String,

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

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl PublicTransport {
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
    pub async fn send<I: Encode, O: Decode>(&self, request: PublicTransportRequest<I, O>) -> Result<O, SendError> {
        let mut url = Url::try_from(request.base_url.as_str())?;
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

        O::decode(http_response).await.map_err(SendError::Other)
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
    fn transform(&self, request: RequestBuilder) -> RequestBuilder;
}

#[derive(Default)]
pub struct EmptyBody;

impl TransportBody for EmptyBody {
    fn transform(&self, request: RequestBuilder) -> RequestBuilder {
        request
    }
}

#[derive(Debug, derive_more::From)]
pub struct UrlEncodedBody<T: Serialize + Sized>(T);

impl<T: Serialize + Sized> TransportBody for UrlEncodedBody<T> {
    fn transform(&self, request: RequestBuilder) -> RequestBuilder {
        request.form(&self.0)
    }
}

#[derive(From)]
pub struct EncodedProtobufBody<T: prost::Message>(T);

impl<T: prost::Message> TransportBody for EncodedProtobufBody<T> {
    fn transform(&self, request: RequestBuilder) -> RequestBuilder {
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
    path: String,

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

        let mut http_request = http_client.request(request.method, url).headers(headers);

        http_request = request.body.transform(http_request);

        let response = http_request.send().await?;

        steamlang::ensure_success(&response)?;
        steamlang::ensure_eresult(&response)?;

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
