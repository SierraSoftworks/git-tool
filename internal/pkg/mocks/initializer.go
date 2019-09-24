package mocks

import "github.com/SierraSoftworks/git-tool/pkg/models"

type Initializer struct {
	MockCalls []struct {
		Function string
		Target   models.Target
	}
	MockError error
}

func (i *Initializer) CreateRepository(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"CreateRepository",
		r,
	})

	return i.MockError
}

func (i *Initializer) CloneRepository(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"CloneRepository",
		r,
	})

	return i.MockError
}

func (i *Initializer) CreateScratchpad(r models.Scratchpad) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"CreateScratchpad",
		r,
	})

	return i.MockError
}
