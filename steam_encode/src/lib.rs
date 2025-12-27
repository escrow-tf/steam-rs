use reqwest::Response;
use reqwest_middleware::RequestBuilder;

pub trait Encode {
    fn encode(&self, request: RequestBuilder) -> RequestBuilder;
}

pub trait Decode
where
    Self: Sized,
{
    fn decode(response: Response) -> impl Future<Output = anyhow::Result<Self>>;
}
