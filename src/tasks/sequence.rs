use super::*;
use std::sync::Arc;
use tracing_batteries::prelude::*;

pub struct Sequence {
    tasks: Vec<Arc<dyn Task + Send + Sync>>,
}

impl Sequence {
    pub fn new(tasks: Vec<Arc<dyn Task + Send + Sync>>) -> Self {
        Self { tasks }
    }
}

#[async_trait]
impl Task for Sequence {
    fn name(&self) -> &'static str {
        "sequence"
    }

    #[tracing::instrument(name = "task:sequence(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &engine::Repo) -> Result<(), engine::Error> {
        for task in self.tasks.iter() {
            let result = task.apply_repo(core, repo).await;
            record_task_event(core, task.as_ref(), "repo", &result);
            result?;
        }

        Ok(())
    }

    #[tracing::instrument(name = "task:sequence(scratchpad)", err, skip(self, core))]
    async fn apply_scratchpad(
        &self,
        core: &Core,
        scratch: &engine::Scratchpad,
    ) -> Result<(), engine::Error> {
        for task in self.tasks.iter() {
            let result = task.apply_scratchpad(core, scratch).await;
            record_task_event(core, task.as_ref(), "scratchpad", &result);
            result?;
        }

        Ok(())
    }
}

/// Records a telemetry event for a task which has just been applied. Only the
/// task's hard-coded [`Task::name`] and the kind of target it was applied to are
/// reported — never anything about the target itself.
fn record_task_event(
    core: &Core,
    task: &(dyn Task + Send + Sync),
    target: &'static str,
    result: &Result<(), engine::Error>,
) {
    // Nested sequences report their child tasks themselves.
    if task.name() == "sequence" {
        return;
    }

    core.analytics().record_event(
        format!("tasks::{}", task.name()),
        [
            (
                "status",
                if result.is_ok() {
                    "succeeded".to_string()
                } else {
                    "failed".to_string()
                },
            ),
            ("target", target.to_string()),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;

    #[tokio::test]
    async fn test_empty_sequence() {
        let seq = Sequence::new(vec![]);
        let repo = get_repo();
        let scratch = get_scratch();
        let core = Core::builder()
            .with_config(Config::from_str("directory: /dev").unwrap())
            .build();

        seq.apply_repo(&core, &repo).await.unwrap();
        seq.apply_scratchpad(&core, &scratch).await.unwrap();
    }

    #[tokio::test]
    async fn test_repo() {
        let task1 = Arc::new(TestTask::default());
        let task2 = Arc::new(TestTask::default());
        let seq = Sequence::new(vec![task1.clone(), task2.clone()]);

        let repo = get_repo();
        let core = Core::builder()
            .with_config(Config::from_str("directory: /dev").unwrap())
            .build();

        seq.apply_repo(&core, &repo).await.unwrap();

        for task in [task1.clone(), task2.clone()] {
            let r = task.ran_repo.lock().await;
            let ran_repo = r.clone().unwrap();
            assert_eq!(ran_repo.get_name(), "git-tool");
        }
    }

    #[tokio::test]
    async fn test_scratchpad() {
        let task1 = Arc::new(TestTask::default());
        let task2 = Arc::new(TestTask::default());
        let seq = Sequence::new(vec![task1.clone(), task2.clone()]);

        let scratch = get_scratch();
        let core = Core::builder()
            .with_config(Config::from_str("directory: /dev").unwrap())
            .build();

        seq.apply_scratchpad(&core, &scratch).await.unwrap();

        for task in [task1.clone(), task2.clone()] {
            let s = task.ran_scratchpad.lock().await;
            let ran_scratch = s.clone().unwrap();
            assert_eq!(ran_scratch.get_name(), "2020w07");
        }
    }

    fn get_repo() -> Repo {
        Repo::new(
            "gh:sierrasoftworks/git-tool",
            std::path::PathBuf::from("/test/github.com/sierrasoftworks/git-tool"),
        )
    }

    fn get_scratch() -> Scratchpad {
        Scratchpad::new("2020w07", std::path::PathBuf::from("/test/scratch/2020w07"))
    }
}
