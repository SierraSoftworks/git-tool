use std::sync::{Arc, RwLock};

use sentry::ClientInitGuard;

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
        let logger = sentry::integrations::log::SentryLogger::new();
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(log::LevelFilter::Debug);

        let raven = sentry::init((
            "https://0787127414b24323be5a3d34767cb9b8@o219072.ingest.sentry.io/1486938",
            sentry::ClientOptions {
                release: Some(version!("git-tool@v").into()),
                default_integrations: true,
                attach_stacktrace: true,
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

        sentry::start_session();

        Self { raven }
    }

    pub fn complete(&self) {
        if !is_enabled() {
            return;
        }

        sentry::end_session_with_status(sentry::protocol::SessionStatus::Exited);
        self.raven.close(None);
    }

    pub fn crash(&self, err: Error) {
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
