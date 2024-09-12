use std::sync::Arc;
use tracing_batteries::prelude::*;

pub fn setup() -> tracing_batteries::Session {
    tracing_batteries::Session::new("git-tool", version!())
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
        .with_battery(
            tracing_batteries::OpenTelemetry::new("https://api.honeycomb.io:443").with_header(
                "x-honeycomb-team",
                #[cfg(debug_assertions)]
                "X6naTEMkzy10PMiuzJKifF",
                #[cfg(not(debug_assertions))]
                "vdf1xcENEju8V0d8ffQq2Y",
            ),
        )
}
