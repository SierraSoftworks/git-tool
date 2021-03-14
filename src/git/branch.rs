use super::git_cmd;
use crate::errors;
use std::{collections::HashSet, path};
use tokio::process::Command;

pub async fn git_current_branch(repo: &path::Path) -> Result<String, errors::Error> {
    info!("Running `git symbolic-ref --short -q HEAD` to get the current branch name");
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

pub async fn git_branches(repo: &path::Path) -> Result<Vec<String>, errors::Error> {
    info!("Running `git for-each-ref --format=%(refname:lstrip=2) refs/heads/` to get the list of branches");
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
        if r.starts_with("remotes/") {
            match r.splitn(3, '/').nth(2) {
                Some(rs) => unique_refs.insert(rs),
                None => unique_refs.insert(r),
            };
        } else {
            unique_refs.insert(r);
        }
    }

    Ok(unique_refs.iter().map(|s| s.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use crate::git::*;
    use crate::tasks::*;
    use path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_get_current_branch() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("github.com/sierrasoftworks/test1", temp.path().into());
        let core = Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        sequence![
            GitInit {},
            GitCheckout { branch: "main" },
            WriteFile {
                path: PathBuf::from("README.md"),
                content: "This is a test file".into(),
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

        let branch = git_current_branch(&repo.get_path())
            .await
            .expect("should be able to get the current branch");
        assert_eq!(branch, "main", "the current branch should be 'main'");
    }

    #[tokio::test]
    async fn test_get_branches() {
        let temp = tempdir().unwrap();
        let repo = Repo::new("github.com/sierrasoftworks/test1", temp.path().into());
        let core = Core::builder()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        sequence![
            GitInit {},
            GitRemote { name: "origin" },
            GitCheckout { branch: "main" },
            WriteFile {
                path: PathBuf::from("README.md"),
                content: "This is a test file".into(),
            },
            GitAdd {
                paths: vec!["README.md"]
            },
            GitCommit {
                message: "Test",
                paths: vec!["README.md"],
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .expect("the repo should have been prepared properly");

        let current_sha = git_rev_parse(&repo.get_path(), "HEAD")
            .await
            .expect("to get the current HEAD SHA");

        assert_ne!(current_sha, "", "the current SHA shouldn't be empty");

        git_update_ref(&repo.get_path(), "refs/heads/test", &current_sha)
            .await
            .unwrap();
        git_update_ref(
            &repo.get_path(),
            "refs/heads/remotes/origin/main",
            &current_sha,
        )
        .await
        .unwrap();
        git_update_ref(
            &repo.get_path(),
            "refs/heads/remotes/origin/test2",
            &current_sha,
        )
        .await
        .unwrap();

        let branch = git_branches(&repo.get_path())
            .await
            .expect("should be able to get the branches list");

        println!("{:?}", branch);

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
            "'test' should be present in the list"
        );
    }
}
