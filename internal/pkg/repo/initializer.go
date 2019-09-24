package repo

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
	"github.com/SierraSoftworks/git-tool/internal/pkg/tasks"
)

type Initializer struct {
}

func (i *Initializer) CreateScratchpad(r models.Scratchpad) error {
	return tasks.Sequence(
		tasks.NewFolder(),
	).ApplyScratchpad(r)
}

func (i *Initializer) CreateRepository(r models.Repo) error {
	return tasks.Sequence(
		tasks.NewFolder(),
		tasks.GitInit(),
		tasks.GitRemote("origin"),
		tasks.SetupRemote(),
	).ApplyRepo(r)
}

func (i *Initializer) CloneRepository(r models.Repo) error {
	return tasks.Sequence(
		tasks.GitClone(),
	).ApplyRepo(r)
}