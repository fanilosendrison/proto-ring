//! GitHub-specific observation of a consumer-bound Authoritative Ref Monotonicity realization.

mod binding;
mod http;
mod observation;

use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmStatus {
    Satisfied,
    Violated,
    Undetermined,
}

impl ArmStatus {
    pub fn serialized(self) -> &'static str {
        match self {
            Self::Satisfied => "SATISFIED",
            Self::Violated => "VIOLATED",
            Self::Undetermined => "UNDETERMINED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArmDiagnosticKind {
    BindingInvalid,
    ProviderObservationUnavailable,
    RequiredProtectionsMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArmDiagnostic {
    pub kind: ArmDiagnosticKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArmObservation {
    pub status: ArmStatus,
    pub diagnostics: Vec<ArmDiagnostic>,
}

pub fn observe(binding_path: &Path) -> ArmObservation {
    let mut transport = http::LiveHttpTransport::new();
    observation::observe_with_transport(binding_path, &mut transport)
}

#[cfg(feature = "conformance-internal")]
pub mod conformance_internal {
    use super::http::{
        HttpFailure as EngineFailure, HttpRequest as EngineRequest, HttpResponse as EngineResponse,
        HttpTransport,
    };
    use super::{ArmObservation, observation};
    use std::path::Path;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct HttpRequest {
        pub url: String,
        pub headers: Vec<(&'static str, &'static str)>,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct HttpResponse {
        pub status: u16,
        pub body: Vec<u8>,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum HttpFailure {
        Network,
        Tls,
        Timeout,
        Io,
    }

    pub fn observe_with_transport(
        binding_path: &Path,
        get: impl FnMut(&HttpRequest) -> Result<HttpResponse, HttpFailure>,
    ) -> ArmObservation {
        let mut transport = InjectedTransport { get };
        observation::observe_with_transport(binding_path, &mut transport)
    }

    struct InjectedTransport<F> {
        get: F,
    }

    impl<F> HttpTransport for InjectedTransport<F>
    where
        F: FnMut(&HttpRequest) -> Result<HttpResponse, HttpFailure>,
    {
        fn get(&mut self, request: &EngineRequest) -> Result<EngineResponse, EngineFailure> {
            let request = HttpRequest {
                url: request.url.clone(),
                headers: request.headers.to_vec(),
            };
            (self.get)(&request)
                .map(|response| EngineResponse {
                    status: response.status,
                    body: response.body,
                })
                .map_err(|failure| match failure {
                    HttpFailure::Network => EngineFailure::Network,
                    HttpFailure::Tls => EngineFailure::Tls,
                    HttpFailure::Timeout => EngineFailure::Timeout,
                    HttpFailure::Io => EngineFailure::Io,
                })
        }
    }
}
