use super::{Resolver, TrueResolver};
use crate::engine::{Core, TempMode, TempTarget};
use tracing_batteries::prelude::*;

/// Resolves a fresh temporary directory target. The [`TempMode`] controls
/// whether the directory is removed ([`TempMode::Cleanup`]) or left in place
/// ([`TempMode::Retain`]) when the target is dropped.
impl Resolver<TempMode, TempTarget> for Core {
    fn resolve(&self, source: TempMode) -> Result<TempTarget, human_errors::Error> {
        self.resolve_with_events(
            source,
            match source {
                TempMode::Cleanup => "cleanup",
                TempMode::Retain => "retain",
            },
        )
    }
}

impl Resolver<TempMode, TempTarget> for TrueResolver {
    #[tracing::instrument(err, skip(self))]
    fn resolve(&self, mode: TempMode) -> Result<TempTarget, human_errors::Error> {
        TempTarget::new(mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Target;

    #[test]
    fn resolves_a_temp_target() {
        let core = Core::builder().with_default_config().build();

        let temp: TempTarget = core.resolve(TempMode::Cleanup).unwrap();
        assert!(temp.get_path().exists());
        temp.close().unwrap();
    }
}
