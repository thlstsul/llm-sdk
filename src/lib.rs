mod api;
mod middleware;

pub use api::*;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use derive_builder::Builder;
use futures_util::{future, StreamExt};
use middleware::RetryMiddleware;
use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use reqwest_tracing::TracingMiddleware;
use schemars::{schema_for, JsonSchema};
use std::time::Duration;
use tracing::error;

const TIMEOUT: u64 = 60;
const MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone, Builder)]
pub struct LlmSdk {
    #[builder(setter(into), default = r#""https://api.openai.com/v1".into()"#)]
    pub(crate) base_url: String,
    #[builder(setter(into))]
    pub(crate) token: String,
    #[allow(dead_code)]
    #[builder(default = "3")]
    pub(crate) max_retries: u32,
    #[builder(setter(skip), default = "self.default_client()")]
    pub(crate) client: ClientWithMiddleware,
}

pub trait IntoRequest {
    fn into_request(self, base_url: &str, client: ClientWithMiddleware) -> RequestBuilder;
}

/// For tool function. If you have a function that you want ChatGPT to call, you shall put
/// all params into a struct and derive schemars::JsonSchema for it. Then you can use
/// `YourStruct::to_schema()` to generate json schema for tools.
pub trait ToSchema: JsonSchema {
    fn to_schema() -> serde_json::Value;
}

impl LlmSdkBuilder {
    // Private helper method with access to the builder struct.
    fn default_client(&self) -> ClientWithMiddleware {
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(self.max_retries.unwrap_or(MAX_RETRIES));
        let m = RetryTransientMiddleware::new_with_policy(retry_policy);
        ClientBuilder::new(reqwest::Client::new())
            // Trace HTTP requests. See the tracing crate to make use of these traces.
            .with(TracingMiddleware::default())
            // Retry failed requests.
            .with(RetryMiddleware::from(m))
            .build()
    }
}

impl LlmSdk {
    pub fn new(token: impl Into<String>) -> Self {
        LlmSdkBuilder::default().token(token).build().unwrap()
    }

    pub fn new_with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        LlmSdkBuilder::default()
            .token(token)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    pub async fn chat_completion(
        &self,
        req: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        assert!(!req.stream.unwrap_or_default());
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<ChatCompletionResponse>().await?)
    }

    pub async fn chat_stream(
        &self,
        req: ChatCompletionRequest,
        mut f: impl FnMut(&ChatStreamResponse),
    ) -> Result<()> {
        assert!(req.stream.unwrap_or_default());
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;

        let mut stream = res
            .bytes_stream()
            .filter(|i| future::ready(i.is_ok()))
            .map(|i| {
                let s = String::from_utf8(i.unwrap().to_vec()).unwrap();
                s.split("\n\n")
                    .map(|ss| ss.strip_prefix("data: "))
                    .filter(|ss| matches!(ss, Some(sss) if !sss.is_empty() && "[DONE]" != *sss))
                    .map(|ss| serde_json::from_str(ss.unwrap()).unwrap())
                    .collect::<Vec<ChatStreamResponse>>()
            });
        while let Some(r) = stream.next().await {
            r.iter().for_each(&mut f);
        }
        Ok(())
    }

    pub async fn create_image(&self, req: CreateImageRequest) -> Result<CreateImageResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json::<CreateImageResponse>().await?)
    }

    pub async fn speech(&self, req: SpeechRequest) -> Result<Bytes> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.bytes().await?)
    }

    pub async fn whisper(&self, req: WhisperRequest) -> Result<WhisperResponse> {
        let is_json = req.response_format == WhisperResponseFormat::Json;
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        let ret = if is_json {
            res.json::<WhisperResponse>().await?
        } else {
            let text = res.text().await?;
            WhisperResponse { text }
        };
        Ok(ret)
    }

    pub async fn embedding(&self, req: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let req = self.prepare_request(req);
        let res = req.send_and_log().await?;
        Ok(res.json().await?)
    }

    fn prepare_request(&self, req: impl IntoRequest) -> RequestBuilder {
        let req = req.into_request(&self.base_url, self.client.clone());
        let req = if self.token.is_empty() {
            req
        } else {
            req.bearer_auth(&self.token)
        };
        req.timeout(Duration::from_secs(TIMEOUT))
    }
}

trait SendAndLog {
    async fn send_and_log(self) -> Result<Response>;
}

impl SendAndLog for RequestBuilder {
    async fn send_and_log(self) -> Result<Response> {
        let res = self.send().await?;
        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            let text = res.text().await?;
            error!("API failed: {}", text);
            return Err(anyhow!("API failed: {}", text));
        }
        Ok(res)
    }
}

impl<T: JsonSchema> ToSchema for T {
    fn to_schema() -> serde_json::Value {
        serde_json::to_value(schema_for!(Self)).unwrap()
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt::init();
}

#[cfg(test)]
lazy_static::lazy_static! {
    static ref SDK: LlmSdk = LlmSdk::new(std::env::var("OPENAI_API_KEY").unwrap());
}
