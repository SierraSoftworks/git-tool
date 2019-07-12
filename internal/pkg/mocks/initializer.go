package mocks

import "github.com/SierraSoftworks/git-tool/pkg/models"

type Initializer struct {
	MockCalls []struct {
		Function string
		Target   models.Target
	}
	MockError error
}

func (i *Initializer) Init(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"Init",
		r,
	})

	return i.MockError
}

func (i *Initializer) Pull(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"Pull",
		r,
	})

	return i.MockError
}

func (i *Initializer) Clone(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"Clone",
		r,
	})

	return i.MockError
}

func (i *Initializer) EnsureRemoteRepo(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"EnsureRemoteRepo",
		r,
	})

	return i.MockError
}

func (i *Initializer) CreateRemoteRepo(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Target   models.Target
	}{
		"CreateRemoteRepo",
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
