use std::borrow::Cow;
use std::sync::Arc;

use tracing_batteries::Session;

/// A shareable handle to the application's telemetry [`Session`] through which
/// the engine records usage events for key operations (tasks, resolutions, web
/// requests, application launches, ...).
///
/// The handle is attached to a [`super::Core`] when the application starts and is
/// reachable anywhere the core is through [`super::Core::analytics`]. When telemetry
/// is disabled at compile time — and within tests, where no session exists — the
/// handle is simply empty and recording an event is a no-op. At runtime, delivery
/// is additionally gated by the session's own enabled flag, which honours the
/// user's `telemetry` feature flag in their configuration file.
#[derive(Clone, Default)]
pub struct Analytics {
    session: Option<Arc<Session>>,
}

impl Analytics {
    /// Creates a handle which records events through the provided telemetry session.
    #[cfg_attr(not(feature = "telemetry"), allow(dead_code))]
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: Some(session),
        }
    }

    /// Creates a handle which records nothing. This is the default for cores built
    /// without an explicit session (tests, telemetry-less builds).
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Records a usage event against the telemetry session.
    ///
    /// Event names identify the operation which was performed and are namespaced
    /// with `::` (for example `commands::list` or `tasks::git-clone`) so that the
    /// events of a session trace group naturally and read intuitively.
    ///
    /// Events **must** be privacy preserving: every segment of the event name has
    /// to come from a hard-coded set (command names, [`crate::tasks::Task::name`]
    /// values, whitelisted phases, ...), property keys are forced to be literals by
    /// this signature, and property *values* must never carry information which
    /// could identify the user or their work — no hostnames, repository or
    /// application names, paths, branch names, or anything else derived from user
    /// input or configuration. Safe values are hard-coded enumerations and data
    /// about the binary itself: exit codes, counts, booleans, release versions,
    /// and the like.
    pub fn record_event<N, P>(&self, name: N, properties: P)
    where
        N: Into<Cow<'static, str>>,
        P: IntoIterator<Item = (&'static str, String)>,
    {
        if let Some(session) = &self.session {
            session.record_event(
                name,
                properties
                    .into_iter()
                    .map(|(key, value)| (key.to_string(), value))
                    .collect(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_handle_is_a_noop() {
        let analytics = Analytics::disabled();
        analytics.record_event("test.event", [("key", "value".to_string())]);
    }
}
