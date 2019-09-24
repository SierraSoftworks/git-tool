package tasks

import (
	"os"

	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/pkg/errors"
)

// NewFolder is responsible for ensuring that a repository has a valid folder
// created for it.
func NewFolder() Task {
	return &newFolder{}
}

// newFolder is responsible for ensuring that a repository has a valid folder
// created for it.
type newFolder struct {
}

// ApplyRepo runs the task against a repository
func (t *newFolder) ApplyRepo(r models.Repo) error {
	return errors.Wrap(os.MkdirAll(r.Path(), os.ModePerm), "repo: unable to create repo directory")
}

// ApplyScratchpad runs the task against a scratchpad
func (t *newFolder) ApplyScratchpad(r models.Scratchpad) error {
	return errors.Wrap(os.MkdirAll(r.Path(), os.ModePerm), "repo: unable to create scratchpad directory")
}
