use std::sync::Arc;
use tracing_batteries::prelude::*;

/// Initializes the telemetry session for this run of Git-Tool.
///
/// The session is returned behind an [`Arc`] so that a handle to it can be shared
/// with the [`crate::engine::Core`] (via [`crate::engine::Analytics`]), allowing usage
/// events to be recorded from anywhere in the application. The `main` function keeps
/// the original reference and hands it back to [`shutdown`] once the run completes.
#[cfg(feature = "telemetry")]
pub fn setup(id: &str) -> Arc<tracing_batteries::Session> {
    Arc::new(session(id))
}

#[cfg(feature = "telemetry")]
fn session(id: &str) -> tracing_batteries::Session {
    tracing_batteries::Session::new("git-tool", version!())
        .with_context(
            "host.environment",
            if cfg!(debug_assertions) {
                "Development"
            } else {
                "Customer"
            },
        )
        .with_battery(tracing_batteries::Sentry::new((
            "https://0787127414b24323be5a3d34767cb9b8@o219072.ingest.sentry.io/1486938",
            sentry::ClientOptions {
                release: Some(version!("git-tool@v").into()),
                #[cfg(debug_assertions)]
                environment: Some("Development".into()),
                #[cfg(not(debug_assertions))]
                environment: Some("Customer".into()),
                default_integrations: true,
                attach_stacktrace: true,
                send_default_pii: false,
                before_send: Some(Arc::new(|mut event| {
                    // Don't send the server name (as it may leak information about the user)
                    event.server_name = None;

                    Some(event)
                })),
                ..Default::default()
            },
        )))
        .with_battery(tracing_batteries::OpenTelemetry::new(
            "https://telemetry.sierrasoftworks.com",
        ))
        .with_battery(
            tracing_batteries::Analytics::new("https://analytics.sierrasoftworks.com")
                .with_session_id(id)
                .without_initial_page(),
        )
}

#[cfg(not(feature = "telemetry"))]
pub fn setup(_id: &str) -> () {
    ()
}

/// Shuts down the telemetry session, flushing any buffered telemetry before the
/// process exits.
///
/// By this point the [`crate::engine::Core`] (and with it every shared handle to the
/// session) has been dropped, so reclaiming exclusive ownership from the [`Arc`] is
/// expected to succeed; if some handle does still exist we skip the final flush
/// rather than blocking or crashing on our way out.
#[cfg(feature = "telemetry")]
pub fn shutdown(session: Arc<tracing_batteries::Session>) {
    match Arc::try_unwrap(session) {
        Ok(session) => session.shutdown(),
        Err(_) => {
            warn!("The telemetry session is still shared elsewhere, skipping the final flush.")
        }
    }
}

#[cfg(not(feature = "telemetry"))]
pub fn shutdown(_session: ()) {}
