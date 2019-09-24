package tasks_test

import "github.com/SierraSoftworks/git-tool/pkg/models"

type TestTask struct {
	OnRepo       func(r models.Repo) error
	OnScratchpad func(s models.Scratchpad) error
}

func (t *TestTask) ApplyRepo(r models.Repo) error {
	if t.OnRepo == nil {
		return nil
	}

	return t.OnRepo(r)
}

func (t *TestTask) ApplyScratchpad(s models.Scratchpad) error {
	if t.OnScratchpad == nil {
		return nil
	}

	return t.OnScratchpad(s)
}
