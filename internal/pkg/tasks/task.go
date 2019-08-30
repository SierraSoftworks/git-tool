package tasks

import "github.com/SierraSoftworks/git-tool/pkg/models"

// A Task is responsible for performing a specific task on a repository
// or scratchpad.
type Task interface {
	ApplyRepo(r models.Repo) error
	ApplyScratchpad(r models.Scratchpad) error
}
