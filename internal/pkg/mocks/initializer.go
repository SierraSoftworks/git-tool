package mocks

import "github.com/SierraSoftworks/git-tool/pkg/models"

type Initializer struct {
	MockCalls []struct {
		Function string
		Repo     models.Repo
	}
	MockError error
}

func (i *Initializer) Init(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"Init",
		r,
	})

	return i.MockError
}

func (i *Initializer) Pull(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"Pull",
		r,
	})

	return i.MockError
}

func (i *Initializer) Clone(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"Clone",
		r,
	})

	return i.MockError
}

func (i *Initializer) EnsureRemoteRepo(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"EnsureRemoteRepo",
		r,
	})

	return i.MockError
}

func (i *Initializer) CreateRemoteRepo(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"CreateRemoteRepo",
		r,
	})

	return i.MockError
}

func (i *Initializer) CreateScratchpad(r models.Repo) error {
	i.MockCalls = append(i.MockCalls, struct {
		Function string
		Repo     models.Repo
	}{
		"CreateScratchpad",
		r,
	})

	return i.MockError
}
