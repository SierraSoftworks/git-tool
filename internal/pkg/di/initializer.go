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
	CreateRepository(r models.Repo) error
	CloneRepository(r models.Repo) error
	CreateScratchpad(r models.Scratchpad) error
}
