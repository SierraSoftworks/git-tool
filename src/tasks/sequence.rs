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
    #[tracing::instrument(name = "task:sequence(repo)", err, skip(self, core))]
    async fn apply_repo(&self, core: &Core, repo: &core::Repo) -> Result<(), core::Error> {
        for task in self.tasks.iter() {
            task.apply_repo(core, repo).await?;
        }

        Ok(())
    }

    #[tracing::instrument(name = "task:sequence(scratchpad)", err, skip(self, core))]
    async fn apply_scratchpad(
        &self,
        core: &Core,
        scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        for task in self.tasks.iter() {
            task.apply_scratchpad(core, scratch).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;

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
