package app

import (
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
)

func getMapper() *repo.Mapper {
	return &repo.Mapper{
		Directory: cfg.DevelopmentDirectory(),
		Services: cfg,
	}
}