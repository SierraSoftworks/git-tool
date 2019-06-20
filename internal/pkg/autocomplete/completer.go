package autocomplete

import (
	"fmt"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/config"
	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

type Completer struct {
	Filter string
	Config config.Config

	repos []models.Repo
}

func NewCompleter(cfg config.Config, filter string) *Completer {
	return &Completer{
		Filter: filter,
		Config: cfg,
	}
}

func (c *Completer) getMapper() *repo.Mapper {
	return &repo.Mapper{
		Directory: c.Config.DevelopmentDirectory(),
		Services:  c.Config,
	}
}

func (c *Completer) getRepos() []models.Repo {
	if c.repos != nil {
		return c.repos
	}

	rs, err := c.getMapper().GetRepos()
	if err != nil {
		return []models.Repo{}
	}

	c.repos = rs
	return rs
}

func (c *Completer) complete(value string) {
	if c.matchesFilter(value) {
		fmt.Println(value)
	}
}

func (c *Completer) matchesFilter(value string) bool {
	return Matches(strings.ToLower(value), strings.ToLower(c.Filter))
}
