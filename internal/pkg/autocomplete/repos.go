package autocomplete

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/repo"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
)

// DefaultServiceRepos will generate autocomplete suggestions for repos hosted on your default service.
func (c *Completer) DefaultServiceRepos() {
	svc := c.Config.GetDefaultService()

	if svc == nil {
		return
	}

	mapper := &repo.Mapper{
		Directory: c.Config.DevelopmentDirectory(),
		Services:  c.Config,
	}

	repos, err := mapper.GetReposForService(svc)
	if err != nil {
		return
	}

	for _, repo := range repos {
		c.complete(repo.FullName())
	}
}

// AllServiceRepos will generate autocomplete suggestions for repos hosted on your all services.
func (c *Completer) AllServiceRepos() {
	mapper := &repo.Mapper{
		Directory: c.Config.DevelopmentDirectory(),
		Services:  c.Config,
	}

	repos, err := mapper.GetRepos()
	if err != nil {
		return
	}

	for _, repo := range repos {
		c.complete(templates.RepoQualifiedName(repo))
	}
}

// DefaultServiceNamespaces will complete the namespace placeholders for the default service.
func (c *Completer) DefaultServiceNamespaces() {
	mapper := &repo.Mapper{
		Directory: c.Config.DevelopmentDirectory(),
		Services:  c.Config,
	}

	svc := c.Config.GetDefaultService()
	if svc == nil {
		return
	}

	repos, err := mapper.GetReposForService(svc)
	if err != nil {
		return
	}

	seen := map[string]struct{}{}

	for _, repo := range repos {
		if _, ok := seen[repo.Namespace()]; ok {
			continue
		}

		seen[repo.Namespace()] = struct{}{}

		c.complete(repo.Namespace())
	}
}

// AllServiceNamespaces will complete the namespace placeholders for a fully qualified service name
func (c *Completer) AllServiceNamespaces() {
	mapper := &repo.Mapper{
		Directory: c.Config.DevelopmentDirectory(),
		Services:  c.Config,
	}

	repos, err := mapper.GetRepos()
	if err != nil {
		return
	}

	seen := map[string]struct{}{}

	for _, repo := range repos {
		if _, ok := seen[repo.Namespace()]; ok {
			continue
		}

		seen[repo.Namespace()] = struct{}{}

		c.complete(fmt.Sprintf("%s/%s/\n", repo.Service().Domain(), repo.Namespace()))
	}
}
