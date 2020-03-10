package tasks

import (
	"strings"

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

	w, err := gr.Worktree()
	if err != nil {
		return errors.Wrap(err, "repo: unable to open git worktree")
	}

	status, err := w.Status()
	if err != nil {
		return errors.Wrap(err, "repo: unable to get status of git worktree")
	}

	if !status.IsClean() {
		return errors.Wrap(err, "usage: cannot change branches, workspace is not clean")
	}

	co := &git.CheckoutOptions{
		Branch: plumbing.NewBranchReferenceName(t.RefName),
		Keep:   t.Keep,
	}

	branchRefs, err := t.getBranchReferences(gr)
	if err != nil {
		return err
	}

	ref := branchRefs[co.Branch.Short()]

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

	refCheck, err := gr.Reference(co.Branch, false)
	if err != nil && err != plumbing.ErrReferenceNotFound {
		return errors.Wrap(err, "repo: unable to validate branch name")
	}

	if refCheck != nil {
		co.Create = false
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

func (t *gitCheckout) getBranchReferences(gr *git.Repository) (map[string]*plumbing.Reference, error) {
	refs, err := gr.References()
	if err != nil {
		return nil, errors.Wrap(err, "repo: unable to gather references")
	}

	branchRefs := map[string]*plumbing.Reference{}

	refs.ForEach(func(r *plumbing.Reference) error {
		if r.Type() == plumbing.SymbolicReference {
			return nil
		}

		if !r.Name().IsBranch() && !r.Name().IsRemote() {
			return nil
		}

		if r.Name().IsTag() || r.Name().IsNote() {
			return nil
		}

		branchRefs[r.Name().Short()] = r

		if r.Name().IsRemote() {
			branchName := strings.SplitN(r.Name().Short(), "/", 2)[1]
			if _, ok := branchRefs[branchName]; !ok {
				branchRefs[branchName] = r
			}
		}

		return nil
	})

	return branchRefs, nil
}
