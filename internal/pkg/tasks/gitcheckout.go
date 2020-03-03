package tasks

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing"
)

// GitCheckout is responsible for checking out a specific branch.
func GitCheckout(ref string, keep bool) Task {
	return &gitCheckout{
		RefName: ref,
		Keep:    keep,
	}
}

// gitCheckout is responsible for running the equivalent of a `git checkout -b` for a repository.
type gitCheckout struct {
	RefName string
	Keep    bool
}

// ApplyRepo runs the task against a repository
func (t *gitCheckout) ApplyRepo(r models.Repo) error {
	gr, err := git.PlainOpen(r.Path())
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git repository")
	}

	co := &git.CheckoutOptions{
		Branch: plumbing.NewBranchReferenceName(t.RefName),
		Keep:   t.Keep,
	}

	refs, err := gr.References()
	if err != nil {
		return errors.Wrap(err, "repo: unable to gather references")
	}

	var ref *plumbing.Reference

	refs.ForEach(func(r *plumbing.Reference) error {
		if r.Type() == plumbing.SymbolicReference {
			return nil
		}

		if r.Name().Short() == t.RefName {
			if ref == nil {
				ref = r
			} else if ref.Name().IsRemote() && !r.Name().IsRemote() {
				ref = r
			}
		}

		return nil
	})

	if ref == nil {
		head, err := gr.Head()
		if err != nil {
			return errors.Wrap(err, "repo: unable to create branch")
		}

		co.Hash = head.Hash()
		co.Create = true
	} else if ref.Name().IsRemote() {
		co.Hash = ref.Hash()
		co.Create = true
	}

	w, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git worktree")
	}

	logrus.WithField("repo", r).Debugf("Checking out branch '%s'", co.Branch.String())
	err = w.Checkout(co)

	if err != nil {
		return errors.Wrap(err, "repo: unable to checkout branch")
	}

	return nil
}

// ApplyScratchpad runs the task against a scratchpad
func (t *gitCheckout) ApplyScratchpad(r models.Scratchpad) error {
	return nil
}
