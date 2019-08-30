package tasks

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"gopkg.in/src-d/go-git.v4"
)

// GitPull is responsible for running the equivalent of a `git pull` operation
// for the default remote (origin).
func GitPull(remoteName string) Task {
	return &gitPull{
		RemoteName: remoteName,
	}
}

// gitPull is responsible for running the equivalent of a `git pull` operation
// for the default remote (origin).
type gitPull struct {
	RemoteName string
}

// ApplyRepo runs the task against a repository
func (t *gitPull) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())

	if err != nil {
		return errors.Wrap(err, "repo: unable to open repository")
	}

	wt, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to get repository worktree")
	}

	remoteName := t.RemoteName
	if remoteName == "" {
		remoteName = "origin"
	}

	err = wt.Pull(&git.PullOptions{
		RemoteName: remoteName,
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to pull repository")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitPull) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
