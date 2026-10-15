use std::time::Duration;

use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

pub(super) const ACCEPT: (&str, &str) = ("Accept", "application/vnd.github+json");
pub(super) const API_VERSION: (&str, &str) = ("X-GitHub-Api-Version", "2026-03-10");
pub(super) const USER_AGENT: (&str, &str) = ("User-Agent", "proto-ring-github-arm-conformance");

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HttpRequest {
    pub url: String,
    pub headers: [(&'static str, &'static str); 3],
}

impl HttpRequest {
    pub fn github(url: String) -> Self {
        Self {
            url,
            headers: [ACCEPT, API_VERSION, USER_AGENT],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HttpFailure {
    Network,
    Tls,
    Timeout,
    Io,
    BodyRead,
}

pub(super) trait HttpTransport {
    fn get(&mut self, request: &HttpRequest) -> Result<HttpResponse, HttpFailure>;
}

pub(super) struct LiveHttpTransport {
    agent: Agent,
}

impl LiveHttpTransport {
    pub fn new() -> Self {
        let tls = TlsConfig::builder()
            .provider(TlsProvider::Rustls)
            .root_certs(RootCerts::WebPki)
            .build();
        let config = Agent::config_builder()
            .https_only(true)
            .timeout_global(Some(Duration::from_secs(10)))
            .http_status_as_error(false)
            .max_redirects(0)
            .proxy(None)
            .tls_config(tls)
            .build();
        Self {
            agent: Agent::new_with_config(config),
        }
    }
}

impl HttpTransport for LiveHttpTransport {
    fn get(&mut self, request: &HttpRequest) -> Result<HttpResponse, HttpFailure> {
        let mut builder = self.agent.get(&request.url);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        let mut response = builder.call().map_err(classify_ureq_failure)?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .read_to_vec()
            .map_err(|_| HttpFailure::BodyRead)?;
        Ok(HttpResponse { status, body })
    }
}

fn classify_ureq_failure(error: ureq::Error) -> HttpFailure {
    match error {
        ureq::Error::Timeout(_) => HttpFailure::Timeout,
        ureq::Error::Io(_) => HttpFailure::Io,
        ureq::Error::Tls(_) => HttpFailure::Tls,
        _ => HttpFailure::Network,
    }
}
