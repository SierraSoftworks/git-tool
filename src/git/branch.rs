use super::git_cmd;
use crate::git::cmd::validate_repo_path_exists;
use itertools::intersperse;
use std::{collections::HashSet, path};
use tokio::process::Command;
use tracing_batteries::prelude::*;

#[allow(dead_code)]
pub async fn git_current_branch(repo: &path::Path) -> Result<String, human_errors::Error> {
    info!("Running `git symbolic-ref --short -q HEAD` to get the current branch name");
    validate_repo_path_exists(repo)?;
    Ok(git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("symbolic-ref")
            .arg("--short")
            .arg("-q")
            .arg("HEAD"),
    )
    .await?
    .trim()
    .to_string())
}

pub async fn git_branches(repo: &path::Path) -> Result<Vec<String>, human_errors::Error> {
    info!(
        "Running `git for-each-ref --format=%(refname:lstrip=2) refs/heads/` to get the list of branches"
    );
    validate_repo_path_exists(repo)?;
    let output = git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("branch")
            .arg("-a")
            .arg("--format=%(refname:lstrip=2)"),
    )
    .await?;

    let refs = output.split_terminator('\n').map(|s| s.trim());

    let mut unique_refs = HashSet::new();
    for r in refs {
        if r.ends_with("/HEAD") {
            continue;
        } else if r.starts_with("origin/") {
            match r.split_once('/').map(|x| x.1) {
                Some(rs) => unique_refs.insert(rs),
                None => unique_refs.insert(r),
            };
        } else {
            unique_refs.insert(r);
        }
    }

    Ok(unique_refs.iter().map(|s| s.to_string()).collect())
}

pub async fn git_branch_delete(repo: &path::Path, name: &str) -> Result<(), human_errors::Error> {
    info!("Running `git branch -D $NAME` to delete branch");
    validate_repo_path_exists(repo)?;
    git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("branch")
            .arg("-D")
            .arg(name),
    )
    .await?;

    Ok(())
}

pub async fn git_default_branch(repo: &path::Path) -> Result<String, human_errors::Error> {
    info!("Running `git symbolic-ref refs/remotes/origin/HEAD` to get the default branch");
    validate_repo_path_exists(repo)?;
    Ok(intersperse(
        git_cmd(
            Command::new("git")
                .current_dir(repo)
                .arg("symbolic-ref")
                .arg("refs/remotes/origin/HEAD"),
        )
        .await?
        .trim()
        .split('/')
        .skip(3),
        "/",
    )
    .collect())
}

pub async fn git_merged_branches(repo: &path::Path) -> Result<Vec<String>, human_errors::Error> {
    info!("Running `git branch --merged` to get the list of merged branches");
    validate_repo_path_exists(repo)?;
    let output = git_cmd(
        Command::new("git")
            .current_dir(repo)
            .arg("branch")
            .arg("--merged"),
    )
    .await?;

    let refs = output
        .split_terminator('\n')
        .filter(|&s| !s.starts_with("* "))
        .map(|s| s.trim().to_string());
    Ok(refs.collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::*;
    use crate::git::*;
    use crate::tasks::*;
    use path::PathBuf;
    use std::path::Path;
    use tempfile::tempdir;

    async fn setup_test_repo(path: &Path) -> (Core, Repo) {
        let repo = Repo::new("gh:sierrasoftworks/test1", path.into());
        let core = Core::builder().with_config_for_dev_directory(path).build();

        sequence![
            GitInit {},
            GitCheckout { branch: "main" },
            WriteFile {
                path: PathBuf::from("README.md"),
                content: "This is a test file",
            },
            GitAdd {
                paths: vec!["README.md"]
            },
            GitCommit {
                message: "Test",
                paths: vec!["README.md"]
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .expect("the repo should have been prepared properly");

        (core, repo)
    }

    #[tokio::test]
    async fn test_get_current_branch() {
        let temp = tempdir().unwrap();
        let (_core, repo) = setup_test_repo(temp.path()).await;

        let branch = git_current_branch(&repo.get_path())
            .await
            .expect("should be able to get the current branch");
        assert_eq!(branch, "main", "the current branch should be 'main'");
    }

    #[tokio::test]
    async fn test_get_branches() {
        let temp = tempdir().unwrap();
        let (_core, repo) = setup_test_repo(temp.path()).await;

        let current_sha = git_rev_parse(&repo.get_path(), "HEAD")
            .await
            .expect("to get the current HEAD SHA");

        assert_ne!(current_sha, "", "the current SHA shouldn't be empty");

        git_update_ref(&repo.get_path(), "refs/heads/test", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/main", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/test2", &current_sha)
            .await
            .unwrap();

        let branch = git_branches(&repo.get_path())
            .await
            .expect("should be able to get the branches list");

        println!("{branch:?}");

        assert!(
            branch.iter().any(|x| x == "main"),
            "'main' should be present in the list"
        );
        assert!(
            branch.iter().any(|x| x == "test"),
            "'test' should be present in the list"
        );
        assert!(
            branch.iter().any(|x| x == "test2"),
            "'test2' should be present in the list"
        );
    }

    #[tokio::test]
    async fn test_get_default_branch() {
        let temp = tempdir().unwrap();
        let (core, repo) = setup_test_repo(temp.path()).await;

        let current_sha = git_rev_parse(&repo.get_path(), "HEAD")
            .await
            .expect("to get the current HEAD SHA");

        assert_ne!(current_sha, "", "the current SHA shouldn't be empty");

        git_update_ref(&repo.get_path(), "refs/remotes/origin/main", &current_sha)
            .await
            .unwrap();

        git_update_ref(&repo.get_path(), "refs/heads/origin/test2", &current_sha)
            .await
            .unwrap();

        WriteFile {
            path: PathBuf::from(".git/refs/remotes/origin/HEAD"),
            content: "ref: refs/remotes/origin/main",
        }
        .apply_repo(&core, &repo)
        .await
        .unwrap();

        let default_branch = git_default_branch(&repo.get_path())
            .await
            .expect("should be able to get the branches list");

        println!("{default_branch:?}");

        assert_eq!(
            default_branch, "main",
            "'main' should be present in the list"
        );
    }

    #[tokio::test]
    async fn test_get_merged_branches() {
        let temp = tempdir().unwrap();
        let (_core, repo) = setup_test_repo(temp.path()).await;

        let current_sha = git_rev_parse(&repo.get_path(), "HEAD")
            .await
            .expect("to get the current HEAD SHA");

        assert_ne!(current_sha, "", "the current SHA shouldn't be empty");

        git_update_ref(&repo.get_path(), "refs/heads/test", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/main", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/heads/test2", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/test2", &current_sha)
            .await
            .unwrap();

        let branches = git_merged_branches(&repo.get_path())
            .await
            .expect("should be able to get the branches list");

        println!("{branches:?}");

        assert!(
            !branches.iter().any(|x| x == "main"),
            "'main' should not be present in the list"
        );
        assert!(
            branches.iter().any(|x| x == "test"),
            "'test' should be present in the list"
        );
        assert!(
            branches.iter().any(|x| x == "test2"),
            "'test2' should be present in the list"
        );
    }

    #[tokio::test]
    async fn test_delete_branch() {
        let temp = tempdir().unwrap();
        let (_core, repo) = setup_test_repo(temp.path()).await;

        let current_sha = git_rev_parse(&repo.get_path(), "HEAD")
            .await
            .expect("to get the current HEAD SHA");

        assert_ne!(current_sha, "", "the current SHA shouldn't be empty");

        git_update_ref(&repo.get_path(), "refs/heads/test", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/main", &current_sha)
            .await
            .unwrap();
        git_update_ref(&repo.get_path(), "refs/remotes/origin/test2", &current_sha)
            .await
            .unwrap();

        git_branch_delete(&repo.get_path(), "test")
            .await
            .expect("should be able to delete the branch");

        let branch = git_branches(&repo.get_path())
            .await
            .expect("should be able to get the branches list");

        println!("{branch:?}");

        assert!(
            branch.iter().any(|x| x == "main"),
            "'main' should be present in the list"
        );
        assert!(
            !branch.iter().any(|x| x == "test"),
            "'test' should not be present in the list"
        );
        assert!(
            branch.iter().any(|x| x == "test2"),
            "'test2' should be present in the list"
        );
    }
}
