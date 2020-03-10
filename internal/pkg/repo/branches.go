package repo

import (
	"strings"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing"
)

// GetBranches returns a list of branches for the provided repository.
func (d *Mapper) GetBranches(r models.Repo) ([]string, error) {
	gr, err := git.PlainOpen(r.Path())

	if err != nil {
		return nil, errors.Wrap(err, "repo: unable to open repository")
	}

	refs, err := gr.References()
	if err != nil {
		return nil, errors.Wrap(err, "repo: failed to get branches")
	}

	branchNameSet := map[string]struct{}{}
	err = refs.ForEach(func(ref *plumbing.Reference) error {
		if !ref.Name().IsBranch() && !ref.Name().IsRemote() {
			return nil
		}

		if ref.Name().IsTag() || ref.Name().IsNote() {
			return nil
		}

		branchNameSet[ref.Name().Short()] = struct{}{}
		if ref.Name().IsRemote() {
			branchNameSet[strings.SplitN(ref.Name().Short(), "/", 2)[1]] = struct{}{}
		}

		return nil
	})

	branchNames := make([]string, len(branchNameSet))

	i := 0
	for k := range branchNameSet {
		branchNames[i] = k
		i = i + 1
	}

	if err != nil {
		return nil, errors.Wrap(err, "repo: failed to get branches")
	}

	return branchNames, nil
}
