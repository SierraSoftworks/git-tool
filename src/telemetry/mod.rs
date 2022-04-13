use opentelemetry_otlp::WithExportConfig;
use sentry::ClientInitGuard;
use std::sync::{Arc, RwLock};
use tracing_subscriber::prelude::*;

use crate::core::Error;

lazy_static! {
    static ref ENABLED: RwLock<bool> = RwLock::new(true);
}

pub fn is_enabled() -> bool {
    ENABLED.read().map(|v| *v).unwrap_or_default()
}

pub fn set_enabled(enable: bool) {
    ENABLED.write().map(|mut v| *v = enable).unwrap_or_default()
}
pub struct Session {
    raven: ClientInitGuard,
}

impl Session {
    pub fn new() -> Self {
        let raven = sentry::init((
            "https://0787127414b24323be5a3d34767cb9b8@o219072.ingest.sentry.io/1486938",
            sentry::ClientOptions {
                release: Some(version!("git-tool@v").into()),
                default_integrations: true,
                attach_stacktrace: true,
                send_default_pii: false,
                before_send: Some(Arc::new(|mut event| {
                    if !is_enabled() {
                        None
                    } else {
                        // Don't send the server name (as it may leak information about the user)
                        event.server_name = None;

                        Some(event)
                    }
                })),
                ..Default::default()
            },
        ));

        let mut tracing_metadata = tonic::metadata::MetadataMap::new();
        tracing_metadata.insert(
            "x-honeycomb-team",
            "fd8BghJ1Qd7xBU9s4ULEBC".parse().unwrap(),
        );

        let mut tls_config = rustls::ClientConfig::new();
        tls_config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        tls_config.set_protocols(&vec!["h2".to_string().into()]);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("https://api.honeycomb.io:443")
                    .with_metadata(tracing_metadata)
                    .with_tls_config(
                        tonic::transport::ClientTlsConfig::new().rustls_client_config(tls_config),
                    ),
            )
            .with_trace_config(opentelemetry::sdk::trace::config().with_resource(
                opentelemetry::sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    "git-tool",
                )]),
            ))
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap();

        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::DEBUG)
            .with(tracing_subscriber::filter::filter_fn(|_metadata| {
                is_enabled()
            }))
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .init();

        sentry::start_session();

        Self { raven }
    }

    pub fn complete(&self) {
        opentelemetry::global::shutdown_tracer_provider();

        if !is_enabled() {
            return;
        }

        sentry::end_session_with_status(sentry::protocol::SessionStatus::Exited);
        self.raven.close(None);
    }

    pub fn crash(&self, err: Error) {
        opentelemetry::global::shutdown_tracer_provider();

        if !is_enabled() {
            return;
        }

        if err.is_system() {
            sentry::capture_error(&err);
        }

        sentry::end_session_with_status(sentry::protocol::SessionStatus::Crashed);
        self.raven.close(None);
    }
}
