package templates

import (
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

type repoContext struct {
	Repo    models.Repo
	Service models.Service
}

var repoTemplates = buildTemplates(map[string]string{
	"repo.qualified": `{{ .Service.Domain}}/{{ .Repo.FullName }}`,
	"repo.short":     `{{ .Service.Domain}}/{{ .Repo.FullName }} ({{ .Repo.Website }})`,
	"repo.full": `
Name:       {{ .Repo.Name }}
Namespace:  {{ .Repo.Namespace }}
Service:    {{ .Service.Domain }}
Path:       {{ .Repo.Path }}

URLs:
 - Website:  {{ .Repo.Website }}
 - Git SSH:  {{ .Repo.GitURL }}
 - Git HTTP: {{ .Repo.HttpURL }}
`,
})

// RepoQualifiedName gets a template which will format the fully qualified name of a repo
func RepoQualifiedName(r models.Repo) string {
	return toString(repoTemplates, "repo.qualified", &repoContext{
		Repo:    r,
		Service: r.Service(),
	})
}

// RepoShortInfo gets a template which renders a detailed summary of a repository's details
func RepoShortInfo(r models.Repo) string {
	return toString(repoTemplates, "repo.short", &repoContext{
		Repo:    r,
		Service: r.Service(),
	})
}

// RepoFullInfo gets a template which renders a detailed summary of a repository's details
func RepoFullInfo(r models.Repo) string {
	return toString(repoTemplates, "repo.full", &repoContext{
		Repo:    r,
		Service: r.Service(),
	})
}
