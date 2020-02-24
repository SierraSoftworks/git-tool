package tasks

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing"
)

// GitCheckout is responsible for checking out a specific branch.
func GitCheckout(ref string) Task {
	return &gitCheckout{
		BranchName: ref,
	}
}

// gitCheckout is responsible for running the equivalent of a `git checkout -b` for a repository.
type gitCheckout struct {
	BranchName string
}

// ApplyRepo runs the task against a repository
func (t *gitCheckout) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git repository")
	}

	branch, err := gr.Branch(t.BranchName)
	if err != nil {
		return errors.Wrap(err, "repo: unable to find branch")
	}

	w, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git worktree")
	}

	logrus.WithField("repo", r).Debugf("Checking out branch '%s'", t.BranchName)
	err = w.Checkout(&git.CheckoutOptions{
		Branch: plumbing.NewBranchReferenceName(branch.Name),
		Keep:   true,
	})

	if err != nil {
		return errors.Wrap(err, "repo: unable to checkout branch")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitCheckout) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
