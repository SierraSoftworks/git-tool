package repo

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/SierraSoftworks/git-tool/internal/pkg/di"
	"github.com/SierraSoftworks/git-tool/pkg/models"
)

// NewRepo creates a new repo object for the given service and repo name
func NewRepo(service models.Service, name string) models.Repo {
	return &repo{
		fullName: name,
		service:  service,
		path:     filepath.Join(di.GetConfig().DevelopmentDirectory(), service.Domain(), filepath.FromSlash(name)),
	}
}

// repo describes a repository object with any necessary properties required by
// Git-Tool.
type repo struct {
	fullName string
	service  models.Service
	path     string
}

func (r *repo) FullName() string {
	return r.fullName
}

func (r *repo) Namespace() string {
	parts := strings.Split(r.fullName, "/")
	return strings.Join(parts[:len(parts)-1], "/")
}

func (r *repo) Name() string {
	parts := strings.Split(r.fullName, "/")
	return parts[len(parts)-1]
}

func (r *repo) Service() models.Service {
	return r.service
}

func (r *repo) Path() string {
	return r.path
}

func (r *repo) Website() string {
	return r.service.Website(r)
}

func (r *repo) HttpURL() string {
	return r.service.HttpURL(r)
}

func (r *repo) GitURL() string {
	return r.service.GitURL(r)
}

func (r *repo) Exists() bool {
	s, err := os.Stat(r.path)
	if err != nil && os.IsNotExist(err) {
		return false
	}

	if err != nil {
		return true
	}

	return s.IsDir()
}

func (r *repo) Valid() bool {
	s, err := os.Stat(filepath.Join(r.path, ".git"))
	if err != nil {
		return false
	}

	return s.IsDir()
}

func (r *repo) TemplateContext() *models.TemplateContext {
	return &models.TemplateContext{
		Repo:    r,
		Target:  r,
		Service: r.Service(),
	}
}
