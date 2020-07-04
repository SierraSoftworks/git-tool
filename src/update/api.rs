use super::release::*;
use crate::{core::Core, errors};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

#[async_trait::async_trait]
pub trait Source<C: Core>: Default + Send + Sync {
    async fn get_releases(&self, core: &C) -> Result<Vec<Release>, errors::Error>;
    async fn get_binary<W: io::Write + Send>(
        &self,
        core: &C,
        release: &Release,
        variant: &ReleaseVariant,
        into: &mut W,
    ) -> Result<(), errors::Error>;
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum UpdatePhase {
    #[serde(rename = "no-update")]
    NoUpdate,
    #[serde(rename = "prepare")]
    Prepare,
    #[serde(rename = "replace")]
    Replace,
    #[serde(rename = "cleanup")]
    Cleanup,
}

impl Default for UpdatePhase {
    fn default() -> Self {
        UpdatePhase::NoUpdate
    }
}

impl ToString for UpdatePhase {
    fn to_string(&self) -> String {
        match self {
            UpdatePhase::NoUpdate => "no-update",
            UpdatePhase::Prepare => "prepare",
            UpdatePhase::Replace => "replace",
            UpdatePhase::Cleanup => "cleanup",
        }
        .to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct UpdateState {
    #[serde(rename = "app", default, skip_serializing_if = "Option::is_none")]
    pub target_application: Option<PathBuf>,

    #[serde(rename = "update", default, skip_serializing_if = "Option::is_none")]
    pub temporary_application: Option<PathBuf>,

    pub phase: UpdatePhase,
}

impl UpdateState {
    pub fn for_phase(&self, phase: UpdatePhase) -> Self {
        UpdateState {
            target_application: self.target_application.clone(),
            temporary_application: self.temporary_application.clone(),
            phase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        assert_eq!(
            serde_json::to_string(&UpdateState {
                target_application: Some(PathBuf::from("/bin/git-tool")),
                temporary_application: Some(PathBuf::from("/tmp/git-tool-update")),
                phase: UpdatePhase::Replace
            })
            .unwrap(),
            r#"{"app":"/bin/git-tool","update":"/tmp/git-tool-update","phase":"replace"}"#
        );

        assert_eq!(
            serde_json::to_string(&UpdateState {
                target_application: None,
                temporary_application: Some(PathBuf::from("/tmp/git-tool-update")),
                phase: UpdatePhase::Cleanup
            })
            .unwrap(),
            r#"{"update":"/tmp/git-tool-update","phase":"cleanup"}"#
        );
    }

    #[test]
    fn test_deserialize() {
        let update = UpdateState {
            target_application: None,
            temporary_application: Some(PathBuf::from("/tmp/git-tool-update")),
            phase: UpdatePhase::Cleanup,
        };

        let deserialized: UpdateState =
            serde_json::from_str(r#"{"update":"/tmp/git-tool-update","phase":"cleanup"}"#).unwrap();
        assert_eq!(deserialized, update);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(UpdatePhase::Prepare.to_string(), "prepare");
        assert_eq!(UpdatePhase::Replace.to_string(), "replace");
        assert_eq!(UpdatePhase::Cleanup.to_string(), "cleanup");
        assert_eq!(UpdatePhase::NoUpdate.to_string(), "no-update");
    }

    #[test]
    fn test_for_phase() {
        let update = UpdateState {
            target_application: Some(PathBuf::from("/bin/git-tool")),
            temporary_application: Some(PathBuf::from("/tmp/git-tool-update")),
            phase: UpdatePhase::Replace,
        };

        let new_update = update.for_phase(UpdatePhase::Cleanup);
        assert_eq!(new_update.target_application, update.target_application);
        assert_eq!(
            new_update.temporary_application,
            update.temporary_application
        );
        assert_eq!(
            update.phase,
            UpdatePhase::Replace,
            "the old update entry should not be modified"
        );
        assert_eq!(
            new_update.phase,
            UpdatePhase::Cleanup,
            "the new update entry should have the correct phase"
        );
    }
}
