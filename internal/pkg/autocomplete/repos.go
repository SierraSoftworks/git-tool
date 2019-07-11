package autocomplete

import (
	"fmt"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/internal/pkg/templates"
)

// RepoAliases will generate autocomplete suggestions for your repo aliases.
func (c *Completer) RepoAliases() {
	aliases := di.GetConfig().GetAliases()

	for alias := range aliases {
		c.complete(alias)
	}
}

// DefaultServiceRepos will generate autocomplete suggestions for repos hosted on your default service.
func (c *Completer) DefaultServiceRepos() {
	svc := di.GetConfig().GetDefaultService()

	if svc == nil {
		return
	}

	repos, err := di.GetMapper().GetReposForService(svc)
	if err != nil {
		return
	}

	for _, repo := range repos {
		c.complete(repo.FullName())
	}
}

// AllScratchpads will generate autocomplete suggestions for scratchpads
func (c *Completer) AllScratchpads() {
	repos, err := di.GetMapper().GetScratchpads()
	if err != nil {
		return
	}

	for _, repo := range repos {
		c.complete(repo.Name())
	}
}

// AllServiceRepos will generate autocomplete suggestions for repos hosted on your all services.
func (c *Completer) AllServiceRepos() {
	for _, repo := range c.getRepos() {
		c.complete(templates.RepoQualifiedName(repo))
	}
}

// DefaultServiceNamespaces will complete the namespace placeholders for the default service.
func (c *Completer) DefaultServiceNamespaces() {
	svc := di.GetConfig().GetDefaultService()
	if svc == nil {
		return
	}

	repos, err := di.GetMapper().GetReposForService(svc)
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
	seen := map[string]struct{}{}

	for _, repo := range c.getRepos() {
		if _, ok := seen[repo.Namespace()]; ok {
			continue
		}

		seen[repo.Namespace()] = struct{}{}

		c.complete(fmt.Sprintf("%s/%s/\n", repo.Service().Domain(), repo.Namespace()))
	}
}
