package di

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

var mapper Mapper

func SetMapper(m Mapper) {
	mapper = m
}

func GetMapper() Mapper {
	return mapper
}

// A Mapper holds the information about a developer's source code folder which
// may contain multiple repositories.
type Mapper interface {
	GetRepos() ([]models.Repo, error)
	EnsureRepo(service models.Service, r models.Repo) error
	GetReposForService(service models.Service) ([]models.Repo, error)
	GetBestRepo(name string) (models.Repo, error)
	GetRepo(name string) (models.Repo, error)
	GetRepoForService(service models.Service, name string) (models.Repo, error)
	GetFullyQualifiedRepo(name string) (models.Repo, error)
	GetCurrentDirectoryRepo() (models.Repo, error)
}