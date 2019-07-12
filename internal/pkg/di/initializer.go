package di

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

var initializer Initializer

func SetInitializer(i Initializer) {
	initializer = i
}

func GetInitializer() Initializer {
	return initializer
}

type Initializer interface {
	Init(r models.Repo) error
	Pull(r models.Repo) error
	Clone(r models.Repo) error
	EnsureRemoteRepo(r models.Repo) error
	CreateRemoteRepo(r models.Repo) error
	CreateScratchpad(r models.Scratchpad) error
}
