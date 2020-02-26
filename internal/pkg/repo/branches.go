package repo

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
	"gopkg.in/src-d/go-git.v4"
	"gopkg.in/src-d/go-git.v4/plumbing"
)

func (d *Mapper) GetBranches(r models.Repo) ([]string, error) {
	gr, err := git.PlainOpen(r.Path())

	if err != nil {
		return nil, errors.Wrap(err, "repo: unable to open repository")
	}

	branches, err := gr.Branches()
	if err != nil {
		return nil, errors.Wrap(err, "repo: failed to get branches")
	}

	branchNames := []string{}
	err = branches.ForEach(func(ref *plumbing.Reference) error {
		branchNames = append(branchNames, ref.Name().Short())
		return nil
	})

	if err != nil {
		return nil, errors.Wrap(err, "repo: failed to get branches")
	}

	return branchNames, nil
}
