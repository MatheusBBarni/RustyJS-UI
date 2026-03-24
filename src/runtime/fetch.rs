use anyhow::{Context as AnyhowContext, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Method};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Duration;

/// Supported HTTP methods for the fetch bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl FetchMethod {
    fn as_reqwest_method(self) -> Method {
        match self {
            Self::Get => Method::GET,
            Self::Post => Method::POST,
            Self::Put => Method::PUT,
            Self::Delete => Method::DELETE,
        }
    }
}

/// Request payload accepted by the transport layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchRequest {
    pub url: String,
    pub method: FetchMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout: Option<Duration>,
}

impl FetchRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: FetchMethod::Get,
            headers: Vec::new(),
            body: None,
            timeout: None,
        }
    }

    pub fn with_method(mut self, method: FetchMethod) -> Self {
        self.method = method;
        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Successful fetch completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchResponse {
    pub request_id: u64,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// Failed fetch completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchError {
    pub request_id: u64,
    pub message: String,
}

/// Completion record emitted by the transport channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchCompletion {
    Response(FetchResponse),
    Error(FetchError),
}

impl FetchCompletion {
    pub fn request_id(&self) -> u64 {
        match self {
            Self::Response(response) => response.request_id,
            Self::Error(error) => error.request_id,
        }
    }
}

/// Async HTTP transport used by the runtime bridge.
///
/// Requests are executed on an internal Tokio runtime and completions are
/// pushed onto an internal thread-safe channel for later polling.
#[derive(Debug)]
pub struct FetchTransport {
    client: Client,
    runtime: tokio::runtime::Runtime,
    completions_tx: Sender<FetchCompletion>,
    completions_rx: Receiver<FetchCompletion>,
    next_request_id: AtomicU64,
}

impl FetchTransport {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("failed to build reqwest client")?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("rustyjs-fetch")
            .build()
            .context("failed to build fetch runtime")?;
        let (completions_tx, completions_rx) = mpsc::channel();

        Ok(Self {
            client,
            runtime,
            completions_tx,
            completions_rx,
            next_request_id: AtomicU64::new(1),
        })
    }

    pub fn submit(&self, request: FetchRequest) -> Result<u64> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let client = self.client.clone();
        let completions_tx = self.completions_tx.clone();
        self.runtime.spawn(async move {
            let completion = execute_request(client, request_id, request).await;
            let _ = completions_tx.send(completion);
        });

        Ok(request_id)
    }

    pub fn try_recv_completion(&self) -> Option<FetchCompletion> {
        match self.completions_rx.try_recv() {
            Ok(completion) => Some(completion),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn drain_completions(&self) -> Vec<FetchCompletion> {
        let mut completions = Vec::new();

        while let Some(completion) = self.try_recv_completion() {
            completions.push(completion);
        }

        completions
    }
}

async fn execute_request(
    client: Client,
    request_id: u64,
    request: FetchRequest,
) -> FetchCompletion {
    let mut builder = client.request(request.method.as_reqwest_method(), &request.url);
    builder = builder.headers(to_header_map(&request.headers));

    if let Some(timeout) = request.timeout {
        builder = builder.timeout(timeout);
    }

    if let Some(body) = request.body {
        builder = builder.body(body);
    }

    match builder.send().await {
        Ok(response) => {
            let status = response.status();
            let status_text = status.to_string();
            let headers = collect_headers(response.headers());

            match response.text().await {
                Ok(body) => FetchCompletion::Response(FetchResponse {
                    request_id,
                    status: status.as_u16(),
                    status_text,
                    headers,
                    body,
                }),
                Err(error) => FetchCompletion::Error(FetchError {
                    request_id,
                    message: error.to_string(),
                }),
            }
        }
        Err(error) => FetchCompletion::Error(FetchError {
            request_id,
            message: error.to_string(),
        }),
    }
}

fn to_header_map(headers: &[(String, String)]) -> HeaderMap {
    let mut map = HeaderMap::new();

    for (name, value) in headers {
        let name = match HeaderName::from_bytes(name.as_bytes()) {
            Ok(name) => name,
            Err(_) => continue,
        };
        let value = match HeaderValue::from_str(value) {
            Ok(value) => value,
            Err(_) => continue,
        };

        map.insert(name, value);
    }

    map
}

fn collect_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_owned(), value.to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_builder_defaults_to_get() {
        let request = FetchRequest::new("https://example.com");

        assert_eq!(request.method, FetchMethod::Get);
        assert!(request.headers.is_empty());
        assert!(request.body.is_none());
        assert!(request.timeout.is_none());
    }

    #[test]
    fn completion_reports_request_id() {
        let completion = FetchCompletion::Error(FetchError {
            request_id: 42,
            message: "boom".to_string(),
        });

        assert_eq!(completion.request_id(), 42);
    }

    #[test]
    fn submit_assigns_request_ids_without_external_runtime() {
        let transport = FetchTransport::new().unwrap();
        let first_id = transport
            .submit(FetchRequest::new("http://127.0.0.1:9"))
            .unwrap();
        let second_id = transport
            .submit(FetchRequest::new("http://127.0.0.1:9"))
            .unwrap();

        assert_eq!(first_id, 1);
        assert_eq!(second_id, 2);
    }
}
