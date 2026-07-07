//! Strongly-typed resolution of Git-Tool's entities.
//!
//! Each of Git-Tool's entity types (applications, repositories, scratchpads,
//! branches, worktrees and temporary directories) has its own submodule here
//! containing the [`Resolver`] implementations for that entity. This keeps the
//! per-entity resolution rules — including which sources they can be resolved
//! from — in one reviewable place per entity.
//!
//! Resolutions which depend on the environment (the filesystem, the current
//! working directory, the current time) are implemented on [`TrueResolver`] and
//! reached through [`super::Core`]'s forwarding implementations, so that tests can
//! substitute a [`MockResolver`]. Resolutions which are pure functions of the
//! configuration (applications, branches, worktrees) are implemented on
//! [`super::Core`] directly.

mod app;
mod branch;
#[cfg(test)]
mod mock;
mod repo;
mod scratchpad;
mod temp;
mod worktree;

use std::sync::Arc;

use super::{Config, Core};

#[cfg(test)]
pub use mock::MockResolver;

/// A strongly-typed, pluggable resolver which converts a source value of type `S`
/// into a resolved target of type `T`.
///
/// It is implemented by [`super::Core`] for each of the entity types Git-Tool
/// understands. Callers write `core.resolve(source)` and the return type drives
/// which resolution is performed; `()` conventionally resolves the entity
/// implied by the current context (the current repository, the current week's
/// scratchpad, the default application, ...).
pub trait Resolver<S, T> {
    fn resolve(&self, source: S) -> Result<T, human_errors::Error>;
}

/// The enumeration counterpart to [`Resolver`]: converts a source value of type
/// `S` into every matching entity of type `T`.
///
/// Keeping enumeration on its own trait (rather than implementing
/// `Resolver<S, Vec<T>>`) makes the intent explicit at the call site —
/// `core.resolve_many(())` reads as "list them all", and `()` conventionally
/// enumerates everything the configuration knows about.
pub trait ResolveMany<S, T> {
    fn resolve_many(&self, source: S) -> Result<Vec<T>, human_errors::Error>;
}

/// The production resolver: it locates entities within the development
/// directory using the configuration, the filesystem, the current working
/// directory and the current time. The per-entity [`Resolver`] implementations
/// live in this module's entity submodules; shared helpers are inherent methods
/// on this type.
pub struct TrueResolver {
    pub(self) config: Arc<Config>,
}

impl TrueResolver {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

impl Core {
    /// Forwards a resolution to the active backend, recording a telemetry event for
    /// it. The [`Resolver`] implementations on [`Core`] which are substitutable in
    /// tests route through this rather than calling the backend directly.
    #[cfg(not(test))]
    pub(crate) fn resolve_with_events<S, T>(&self, source: S) -> Result<T, human_errors::Error>
    where
        TrueResolver: Resolver<S, T>,
    {
        let result = self.resolver.resolve_with(source);
        self.record_resolve_event::<T>(result.is_ok(), "one");
        result
    }

    #[cfg(test)]
    pub(crate) fn resolve_with_events<S, T>(&self, source: S) -> Result<T, human_errors::Error>
    where
        TrueResolver: Resolver<S, T>,
        MockResolver: Resolver<S, T>,
    {
        let result = self.resolver.resolve_with(source);
        self.record_resolve_event::<T>(result.is_ok(), "one");
        result
    }

    /// The [`ResolveMany`] counterpart to [`Core::resolve_with_events`].
    #[cfg(not(test))]
    pub(crate) fn resolve_many_with_events<S, T>(
        &self,
        source: S,
    ) -> Result<Vec<T>, human_errors::Error>
    where
        TrueResolver: ResolveMany<S, T>,
    {
        let result = self.resolver.resolve_many_with(source);
        self.record_resolve_event::<T>(result.is_ok(), "many");
        result
    }

    #[cfg(test)]
    pub(crate) fn resolve_many_with_events<S, T>(
        &self,
        source: S,
    ) -> Result<Vec<T>, human_errors::Error>
    where
        TrueResolver: ResolveMany<S, T>,
        MockResolver: ResolveMany<S, T>,
    {
        let result = self.resolver.resolve_many_with(source);
        self.record_resolve_event::<T>(result.is_ok(), "many");
        result
    }

    /// Records a telemetry event for a resolution which has just been performed.
    /// Only the kind of entity being resolved and the outcome are reported — never
    /// the source it was resolved from or the entity it resolved to.
    pub(crate) fn record_resolve_event<T>(&self, success: bool, scope: &'static str) {
        self.analytics().record_event(
            format!("resolve::{}", entity_name::<T>()),
            [
                (
                    "status",
                    if success {
                        "succeeded".to_string()
                    } else {
                        "failed".to_string()
                    },
                ),
                ("scope", scope.to_string()),
            ],
        );
    }
}

/// The short, lowercased name of the entity type being resolved (for example
/// `repo`), derived from the Rust type name — hard-coded by construction, and
/// therefore safe to report.
fn entity_name<T>() -> String {
    std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("unknown")
        .to_lowercase()
}

/// The resolver implementation backing a [`super::Core`]: the production
/// [`TrueResolver`], or a [`MockResolver`] within tests.
pub enum ResolverBackend {
    True(TrueResolver),
    #[cfg(test)]
    Mock(MockResolver),
}

impl ResolverBackend {
    /// Dispatches a resolution to whichever backend is active. [`super::Core`]'s
    /// forwarding [`Resolver`] implementations call this for every resolution
    /// which should be substitutable in tests.
    #[cfg(not(test))]
    pub(crate) fn resolve_with<S, T>(&self, source: S) -> Result<T, human_errors::Error>
    where
        TrueResolver: Resolver<S, T>,
    {
        match self {
            ResolverBackend::True(resolver) => resolver.resolve(source),
        }
    }

    #[cfg(test)]
    pub(crate) fn resolve_with<S, T>(&self, source: S) -> Result<T, human_errors::Error>
    where
        TrueResolver: Resolver<S, T>,
        MockResolver: Resolver<S, T>,
    {
        match self {
            ResolverBackend::True(resolver) => resolver.resolve(source),
            ResolverBackend::Mock(mock) => mock.resolve(source),
        }
    }

    /// The [`ResolveMany`] counterpart to [`ResolverBackend::resolve_with`].
    #[cfg(not(test))]
    pub(crate) fn resolve_many_with<S, T>(&self, source: S) -> Result<Vec<T>, human_errors::Error>
    where
        TrueResolver: ResolveMany<S, T>,
    {
        match self {
            ResolverBackend::True(resolver) => resolver.resolve_many(source),
        }
    }

    #[cfg(test)]
    pub(crate) fn resolve_many_with<S, T>(&self, source: S) -> Result<Vec<T>, human_errors::Error>
    where
        TrueResolver: ResolveMany<S, T>,
        MockResolver: ResolveMany<S, T>,
    {
        match self {
            ResolverBackend::True(resolver) => resolver.resolve_many(source),
            ResolverBackend::Mock(mock) => mock.resolve_many(source),
        }
    }
}
