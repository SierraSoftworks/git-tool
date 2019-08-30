package tasks

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"gopkg.in/src-d/go-git.v4"
)

// GitInit is responsible for running the equivalent of a `git init` operation for
// a repository.
func GitInit() Task {
	return &gitInit{}
}

// gitInit is responsible for running the equivalent of a `git init` operation for
// a repository.
type gitInit struct {
}

// ApplyRepo runs the task against a repository
func (t *gitInit) ApplyRepo(r models.Repo) error {
	_, err := git.PlainInit(r.Path(), false)
	if err != nil && err != git.ErrRepositoryAlreadyExists {
		return errors.Wrap(err, "repo: unable to initialize git repository")
	}
	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitInit) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
