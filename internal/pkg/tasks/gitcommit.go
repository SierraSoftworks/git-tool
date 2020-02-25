package tasks

import (
	"time"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing/object"
)

// GitCommit is responsible for committing changes within the working directory.
func GitCommit(message string, globs ...string) Task {
	return &gitCommit{
		Message: message,
		Globs:   globs,
	}
}

// gitCommit is responsible for running the equivalent of a `git commit -a -m` for a repository.
type gitCommit struct {
	Message string
	Globs   []string
}

// ApplyRepo runs the task against a repository
func (t *gitCommit) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git repository")
	}

	w, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git worktree")
	}

	for _, glob := range t.Globs {
		err := w.AddGlob(glob)
		if err != nil {
			return errors.Wrap(err, "repo: unable to stage files")
		}
	}

	logrus.WithField("repo", r).Debugf("Committing changes")
	_, err = w.Commit(t.Message, &git.CommitOptions{
		Author: &object.Signature{
			Name:  "Git Tool",
			Email: "contact@sierrasoftworks.com",
			When:  time.Now(),
		},
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to commit changes")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitCommit) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
